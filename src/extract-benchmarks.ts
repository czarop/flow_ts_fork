import { constants } from 'fs';
import { access, readdir, readFile, stat, writeFile } from 'fs/promises';
import { join } from 'path';

interface CriterionEstimates {
	mean: {
		point_estimate: number;
		standard_error: number;
	};
	median: {
		point_estimate: number;
	};
	std_dev: {
		point_estimate: number;
	};
}

interface BenchmarkData {
	name: string;
	crate: string;
	mean: number;
	stddev: number;
	unit: string;
	group: string;
	description: string;
	tags: string[];
	plot_path?: string;
}

interface BenchmarkGroup {
	name: string;
	description: string;
	tags: string[];
	benchmark_count: number;
	lines_plot?: string;
	violin_plot?: string;
}

interface BenchmarkMetadata {
	app_version: string;
	benchmark_date: string;
	generated_by: string;
}

function inferCrateFromPath(pathParts: string[]): string {
	// Try to infer crate from benchmark path/name
	// Common patterns: flow-fcs benchmarks might have "fcs" or "dataframe" in name
	// flow-plots might have "plot", "density", "render" in name
	// flow-gates might have "gate", "gating" in name
	
	const pathStr = pathParts.join(' ').toLowerCase();
	
	if (pathStr.includes('dataframe') || pathStr.includes('fcs') || pathStr.includes('metadata') || 
	    pathStr.includes('parsing') || pathStr.includes('alignment') || pathStr.includes('delimiter')) {
		return 'flow-fcs';
	}
	if (pathStr.includes('density') || pathStr.includes('plot') || pathStr.includes('render') || 
	    pathStr.includes('colormap') || pathStr.includes('xy')) {
		return 'flow-plots';
	}
	if (pathStr.includes('gate') || pathStr.includes('gating') || pathStr.includes('filter')) {
		return 'flow-gates';
	}
	
	// Default to flow-fcs as it's likely the most common
	return 'flow-fcs';
}

async function findEstimates(dir: string, benchmarks: BenchmarkData[], seen: Set<string>, crate?: string) {
	try {
		const entries = await readdir(dir, { withFileTypes: true });

		for (const entry of entries) {
			const fullPath = join(dir, entry.name);

			if (entry.isDirectory()) {
				// Recursively search subdirectories
				await findEstimates(fullPath, benchmarks, seen, crate);
			} else if (entry.name === 'estimates.json') {
				// Found an estimates file - extract benchmark name from path
				const pathParts = dir.split('/');
				const criterionIndex = pathParts.lastIndexOf('criterion');

				if (criterionIndex === -1) continue;

				// Get benchmark path after "criterion" directory
				const benchmarkPath = pathParts.slice(criterionIndex + 1);

				// Skip if in "report" or "change" directory (Criterion change detection)
				if (benchmarkPath.includes('report') || benchmarkPath.includes('change')) continue;

				// Remove "base" or "new" from the end
				const cleanPath = benchmarkPath.filter((p) => p !== 'base' && p !== 'new');

				// Create benchmark name
				const benchmarkName = cleanPath.join(' / ');

				// Skip duplicates (base and new versions)
				if (seen.has(benchmarkName)) continue;
				seen.add(benchmarkName);

				try {
					const estimatesData = await readFile(fullPath, 'utf-8');
					const estimates: CriterionEstimates = JSON.parse(estimatesData);

					const group = cleanPath[0] || 'general';
					const { description, tags } = getBenchmarkMetadata(benchmarkName, group);

					// Infer crate if not provided
					const benchmarkCrate = crate || inferCrateFromPath(cleanPath);

					// Find the best available plot (try violin, pdf, then typical)
					const plotPath = await findAvailablePlot(cleanPath);

					benchmarks.push({
						name: benchmarkName,
						crate: benchmarkCrate,
						mean: estimates.mean.point_estimate,
						stddev: estimates.std_dev.point_estimate,
						unit: 'ns',
						group,
						description,
						tags,
						plot_path: plotPath
					});
				} catch (err) {
					console.warn(`Failed to parse ${fullPath}:`, err);
				}
			}
		}
	} catch (err) {
		// Directory doesn't exist or can't be read - that's okay
	}
}

async function findAvailablePlot(cleanPath: string[]): Promise<string | undefined> {
	const basePath = cleanPath.join('/');
	const plotTypes = ['violin.svg', 'pdf.svg', 'typical.svg'];

	for (const plotType of plotTypes) {
		const plotPath = `/criterion-plots/${basePath}/report/${plotType}`;
		const fsPath = join(process.cwd(), 'static', plotPath.slice(1)); // Remove leading /

		try {
			await access(fsPath, constants.R_OK);
			return plotPath; // File exists and is readable
		} catch {
			// File doesn't exist, try next plot type
		}
	}

	return undefined; // No plot found
}

async function extractGroups(
	benchmarks: BenchmarkData[],
	_criterionDir: string
): Promise<BenchmarkGroup[]> {
	// Group by crate first, then by group name
	const crateGroupMap = new Map<string, Map<string, BenchmarkData[]>>();

	for (const benchmark of benchmarks) {
		if (!crateGroupMap.has(benchmark.crate)) {
			crateGroupMap.set(benchmark.crate, new Map());
		}
		const groupMap = crateGroupMap.get(benchmark.crate)!;
		
		if (!groupMap.has(benchmark.group)) {
			groupMap.set(benchmark.group, []);
		}
		groupMap.get(benchmark.group)!.push(benchmark);
	}

	const groups: BenchmarkGroup[] = [];

	// Only include groups with multiple benchmarks (these have throughput plots)
	for (const [crate, groupMap] of crateGroupMap) {
		for (const [groupName, groupBenchmarks] of groupMap) {
			if (groupBenchmarks.length < 2) continue;

			const { description, tags } = getBenchmarkMetadata('', groupName);

			// Check for group-level plots
			const linesPlot = await findGroupPlot(groupName, 'lines.svg');
			const violinPlot = await findGroupPlot(groupName, 'violin.svg');

			// Only add groups that have at least one plot
			if (linesPlot || violinPlot) {
				groups.push({
					name: `${crate}/${groupName}`, // Include crate in group name
					description,
					tags,
					benchmark_count: groupBenchmarks.length,
					lines_plot: linesPlot,
					violin_plot: violinPlot
				});
			}
		}
	}

	return groups.sort((a, b) => a.name.localeCompare(b.name));
}

async function extractMetadata(criterionDir: string): Promise<BenchmarkMetadata> {
	// Read package.json for app version
	let app_version = 'unknown';
	try {
		const packageJsonPath = join(process.cwd(), 'package.json');
		const packageJson = JSON.parse(await readFile(packageJsonPath, 'utf-8'));
		app_version = packageJson.version || 'unknown';
	} catch {
		// Ignore errors, use default
	}

	// Get the most recent benchmark file timestamp
	let benchmark_date = 'unknown';
	try {
		const stats = await stat(criterionDir);
		benchmark_date = stats.mtime.toISOString();
	} catch {
		// Try to find any report file for timestamp
		try {
			const reportDirs = await readdir(criterionDir);
			let latestTime = 0;
			let latestDate = 'unknown';

			for (const dir of reportDirs) {
				try {
					const reportPath = join(criterionDir, dir, 'report');
					const stats = await stat(reportPath);
					if (stats.mtime.getTime() > latestTime) {
						latestTime = stats.mtime.getTime();
						latestDate = stats.mtime.toISOString();
					}
				} catch {
					// Try individual files in the directory
					try {
						const files = await readdir(join(criterionDir, dir));
						for (const file of files) {
							if (file.endsWith('.html') || file.endsWith('.svg')) {
								const filePath = join(criterionDir, dir, file);
								const stats = await stat(filePath);
								if (stats.mtime.getTime() > latestTime) {
									latestTime = stats.mtime.getTime();
									latestDate = stats.mtime.toISOString();
								}
							}
						}
					} catch {
						// Ignore individual file errors
					}
				}
			}
			benchmark_date = latestDate;
		} catch {
			// Use current time as fallback
			benchmark_date = new Date().toISOString();
		}
	}

	return {
		app_version,
		benchmark_date,
		generated_by: 'Criterion.rs'
	};
}

async function findGroupPlot(groupName: string, plotType: string): Promise<string | undefined> {
	const plotPath = `/criterion-plots/${groupName}/report/${plotType}`;
	const fsPath = join(process.cwd(), 'static', plotPath.slice(1));

	try {
		await access(fsPath, constants.R_OK);
		return plotPath;
	} catch {
		return undefined;
	}
}

async function extractBenchmarks() {
	// Check workspace target first, then individual crate targets
	const workspaceCriterionDir = join(process.cwd(), 'target', 'criterion');
	const benchmarksOutputPath = join(process.cwd(), 'static', 'benchmarks.json');
	const groupsOutputPath = join(process.cwd(), 'static', 'benchmark-groups.json');
	const metadataOutputPath = join(process.cwd(), 'static', 'benchmark-metadata.json');

	const benchmarks: BenchmarkData[] = [];
	const seen = new Set<string>();

	try {
		// Try workspace target first (most common in Cargo workspaces)
		try {
			await findEstimates(workspaceCriterionDir, benchmarks, seen);
		} catch {
			// Workspace target doesn't exist, try individual crates
			const crates = ['flow-fcs', 'flow-plots', 'flow-gates'];
			for (const crate of crates) {
				const crateCriterionDir = join(process.cwd(), crate, 'target', 'criterion');
				try {
					await findEstimates(crateCriterionDir, benchmarks, seen, crate);
				} catch {
					// Crate target doesn't exist - that's okay
				}
			}
		}

		// Sort by crate, then by name
		benchmarks.sort((a, b) => {
			if (a.crate !== b.crate) {
				return a.crate.localeCompare(b.crate);
			}
			return a.name.localeCompare(b.name);
		});

		// Extract groups (benchmarks with multiple entries sharing the same group)
		// Group by crate first, then by group name
		const groups = await extractGroups(benchmarks, workspaceCriterionDir);

		// Extract metadata
		const metadata = await extractMetadata(workspaceCriterionDir);

		// Organize benchmarks by crate
		const benchmarksByCrate: Record<string, BenchmarkData[]> = {};
		for (const benchmark of benchmarks) {
			if (!benchmarksByCrate[benchmark.crate]) {
				benchmarksByCrate[benchmark.crate] = [];
			}
			benchmarksByCrate[benchmark.crate].push(benchmark);
		}

		// Write outputs
		await writeFile(benchmarksOutputPath, JSON.stringify(benchmarks, null, 2));
		await writeFile(groupsOutputPath, JSON.stringify(groups, null, 2));
		await writeFile(metadataOutputPath, JSON.stringify(metadata, null, 2));

		console.log(`✅ Extracted ${benchmarks.length} benchmarks to ${benchmarksOutputPath}`);
		console.log(`✅ Extracted ${groups.length} benchmark groups to ${groupsOutputPath}`);
		console.log(`✅ Extracted metadata to ${metadataOutputPath}`);
		
		// Log by crate
		for (const [crate, crateBenchmarks] of Object.entries(benchmarksByCrate)) {
			console.log(`\n   [${crate}] (${crateBenchmarks.length} benchmarks):`);
			crateBenchmarks.forEach((b) => {
				const time = formatTime(b.mean);
				console.log(`     - ${b.name}: ${time}`);
			});
		}
	} catch (err) {
		console.error('❌ Failed to extract benchmarks:', err);
		console.error("\nMake sure you've run benchmarks first: cargo bench --package flow-fcs");
		process.exit(1);
	}
}

function formatTime(ns: number): string {
	if (ns > 1_000_000_000) return `${(ns / 1_000_000_000).toFixed(2)} s`;
	if (ns > 1_000_000) return `${(ns / 1_000_000).toFixed(2)} ms`;
	if (ns > 1_000) return `${(ns / 1_000).toFixed(2)} µs`;
	return `${ns.toFixed(2)} ns`;
}

function getBenchmarkMetadata(
	_name: string,
	group: string
): { description: string; tags: string[] } {
	// Map benchmark names to descriptions and tags
	const metadata: Record<string, { description: string; tags: string[] }> = {
		alignment_detection: {
			description: 'Tests data alignment detection for zero-copy parsing optimization',
			tags: ['performance', 'parsing', 'data structure']
		},
		column_access: {
			description: 'Compares columnar data access performance between ndarray and polars',
			tags: ['performance', 'data structure', 'polars', 'benchmarks']
		},
		delimiter_search: {
			description: 'Compares byte-by-byte vs SIMD (memchr) delimiter search in FCS files',
			tags: ['performance', 'parsing', 'simd', 'optimization']
		},
		density_building: {
			description:
				'Measures density map building with different strategies (array vs parallel vs mutex)',
			tags: ['performance', 'density', 'benchmarks', 'parallelization', 'optimization']
		},
		density_calculation: {
			description: 'Compares old overplotting vs new optimized density calculation',
			tags: ['performance', 'density', 'optimization', 'benchmarks']
		},
		f32_parsing: {
			description: 'Compares byteorder vs bytemuck for f32 parsing from FCS binary data',
			tags: ['performance', 'parsing', 'data structure', 'optimization']
		},
		filtering: {
			description: 'Compares row filtering performance between ndarray and polars',
			tags: ['performance', 'data structure', 'polars', 'optimization']
		},
		get_xy_pairs: {
			description: 'Measures XY coordinate pair extraction for plotting',
			tags: ['performance', 'plotting', 'data structure']
		},
		log_transform: {
			description: 'Compares sequential vs parallel arcsinh transform for density values',
			tags: ['performance', 'transforms', 'parallelization', 'benchmarks']
		},
		memory_allocation: {
			description: 'Compares memory allocation strategies for density maps',
			tags: ['performance', 'memory', 'optimization', 'data structure']
		},
		metadata_parsing: {
			description: 'Measures FCS metadata keyword parsing performance',
			tags: ['performance', 'parsing', 'metadata', 'fcs']
		},
		parallel_overhead_threshold: {
			description: 'Determines when parallel processing overhead is worth it vs sequential',
			tags: ['performance', 'parallelization', 'benchmarks', 'optimization']
		},
		dataframe_parsing: {
			description: 'Compares parallel vs sequential data parsing strategies for FCS files',
			tags: ['performance', 'parsing', 'parallelization', 'fcs', 'dataframe']
		},
		statistics: {
			description: 'Compares statistical calculation performance between ndarray and polars',
			tags: ['performance', 'data structure', 'polars', 'benchmarks']
		},
		zero_copy_slice_access: {
			description: 'Measures zero-copy data slice access performance',
			tags: ['performance', 'data structure', 'optimization', 'memory']
		}
	};

	// Try to match group first, then fall back to defaults
	const meta = metadata[group] || {
		description: `Performance benchmark for ${group}`,
		tags: ['performance', 'benchmarks']
	};

	return meta;
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
	extractBenchmarks();
}

export { extractBenchmarks };

import { readdir, readFile, writeFile } from 'fs/promises';
import * as yaml from 'js-yaml';
import { join } from 'path';

interface SearchIndexItem {
	title: string;
	href: string;
	crate: string;
	excerpt: string;
	tags?: string[];
}

interface Frontmatter {
	title?: string;
	crate?: string;
	tags?: string[];
	[key: string]: unknown;
}

async function buildSearchIndex() {
	const routesDir = join(process.cwd(), 'src', 'routes');
	const outputPath = join(process.cwd(), 'static', 'search-index.json');

	const searchIndex: SearchIndexItem[] = [];

	// Process crate directories (flow-fcs, flow-plots, flow-gates)
	const crates = ['flow-fcs', 'flow-plots', 'flow-gates'];

	for (const crate of crates) {
		const crateDir = join(routesDir, crate);
		try {
			await processDirectory(crateDir, `/${crate}`, searchIndex, crate);
		} catch {
			// Crate directory doesn't exist yet - that's okay
			console.log(`   ℹ️  Skipping ${crate} (directory doesn't exist yet)`);
		}
	}

	// Also process guides if they exist
	const guidesDir = join(routesDir, 'guides');
	try {
		await processDirectory(guidesDir, '/guides', searchIndex, 'guides');
	} catch {
		// Guides directory doesn't exist - that's okay
	}

	// Sort by crate and title
	searchIndex.sort((a, b) => {
		if (a.crate !== b.crate) {
			return a.crate.localeCompare(b.crate);
		}
		return a.title.localeCompare(b.title);
	});

	await writeFile(outputPath, JSON.stringify(searchIndex, null, 2));

	console.log(`✅ Built search index with ${searchIndex.length} entries`);
	searchIndex.forEach((item) => {
		console.log(`   - [${item.crate}] ${item.title}`);
	});
}

async function processDirectory(
	dirPath: string,
	urlPath: string,
	searchIndex: SearchIndexItem[],
	defaultCrate: string
) {
	const entries = await readdir(dirPath, { withFileTypes: true });

	for (const entry of entries) {
		const fullPath = join(dirPath, entry.name);

		if (entry.isDirectory()) {
			// Recursively process subdirectories
			await processDirectory(fullPath, `${urlPath}/${entry.name}`, searchIndex, defaultCrate);
		} else if (entry.name === '+page.svx' || entry.name === '+page.md') {
			// Process SvelteKit page files
			const content = await readFile(fullPath, 'utf-8');
			const { frontmatter, excerpt } = parseFrontmatter(content);

			if (frontmatter.title) {
				// Use crate from frontmatter, or infer from path, or use default
				let crate = frontmatter.crate;
				if (!crate) {
					// Infer crate from URL path
					if (urlPath.startsWith('/flow-fcs')) crate = 'flow-fcs';
					else if (urlPath.startsWith('/flow-plots')) crate = 'flow-plots';
					else if (urlPath.startsWith('/flow-gates')) crate = 'flow-gates';
					else crate = defaultCrate;
				}

				const href = urlPath || '/';

				searchIndex.push({
					title: frontmatter.title,
					href,
					crate,
					excerpt,
					tags: frontmatter.tags
				});
			}
		}
	}
}

function parseFrontmatter(content: string): {
	frontmatter: Frontmatter;
	excerpt: string;
} {
	const frontmatterRegex = /^---\s*\n([\s\S]*?)\n---\s*\n([\s\S]*)/;
	const match = content.match(frontmatterRegex);

	if (!match) {
		return {
			frontmatter: {},
			excerpt: content.slice(0, 200).replace(/\n/g, ' ').trim()
		};
	}

	const frontmatterText = match[1];
	const bodyText = match[2];

	// Parse YAML frontmatter using js-yaml for better compatibility
	let frontmatter: Frontmatter = {};
	try {
		frontmatter = yaml.load(frontmatterText) as Frontmatter;
	} catch (error) {
		console.warn('Failed to parse YAML frontmatter:', error);
		frontmatter = {};
	}

	// Extract excerpt from body (first paragraph)
	const firstParagraph = bodyText
		.split('\n\n')[0]
		.replace(/^#+\s+/, '')
		.replace(/\n/g, ' ')
		.trim()
		.slice(0, 200);

	return {
		frontmatter,
		excerpt: firstParagraph
	};
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
	buildSearchIndex();
}

export { buildSearchIndex };

import { exec } from "child_process";
import { readdir, readFile, writeFile } from "fs/promises";
import { join } from "path";
import { promisify } from "util";
import * as yaml from "js-yaml";

const execAsync = promisify(exec);

interface FeatureHistory {
  title: string;
  date_added: string;
  crate: string;
  author: string;
  github_username?: string;
  excerpt: string;
  href: string;
  commit_hash?: string;
  impact?: Array<{
    metric: string;
    before: string;
    after: string;
    improvement: string;
  }>;
  tags?: string[];
}

interface Frontmatter {
  title?: string;
  date?: string;
  author?: string;
  crate?: string;
  tags?: string[];
  impact?: any;
  [key: string]: any;
}

async function extractFeatureHistory(): Promise<void> {
  const routesDir = join(process.cwd(), "src", "routes");
  const outputPath = join(process.cwd(), "static", "feature-history.json");

  console.log(`Looking for .svx files in: ${routesDir}`);

  const featureHistory: FeatureHistory[] = [];

  try {
    // Get all .svx files in crate directories
    const crates = ["flow-fcs", "flow-plots", "flow-gates"];
    const files: string[] = [];
    
    for (const crate of crates) {
      const crateDir = join(routesDir, crate);
      try {
        const crateFiles = await getAllSvxFiles(crateDir);
        files.push(...crateFiles);
      } catch {
        // Crate directory doesn't exist yet - that's okay
        console.log(`   ℹ️  Skipping ${crate} (directory doesn't exist yet)`);
      }
    }
    
    // Also check guides if they exist
    const guidesDir = join(routesDir, "guides");
    try {
      const guideFiles = await getAllSvxFiles(guidesDir);
      files.push(...guideFiles);
    } catch {
      // Guides directory doesn't exist - that's okay
    }

    for (const file of files) {
      try {
        const content = await readFile(file, "utf-8");
        const { frontmatter, excerpt } = parseFrontmatter(content);

        if (!frontmatter.title || !frontmatter.date) {
          console.warn(
            `Skipping ${file}: missing title or date in frontmatter`,
          );
          continue;
        }

        // Generate href from file path
        // Extract crate and path from file location
        const relativePath = file.replace(routesDir, "");
        const pathParts = relativePath.split("/").filter(Boolean);
        
        // Remove "+page.svx" from the end
        const cleanPath = pathParts.slice(0, -1).join("/");
        const href = "/" + cleanPath;
        
        // Infer crate from path if not in frontmatter
        let crate = frontmatter.crate;
        if (!crate) {
          if (pathParts[0] === "flow-fcs") crate = "flow-fcs";
          else if (pathParts[0] === "flow-plots") crate = "flow-plots";
          else if (pathParts[0] === "flow-gates") crate = "flow-gates";
          else crate = "guides";
        }

        // Extract GitHub username from author field
        let github_username: string | undefined;
        if (frontmatter.author) {
          const usernameMatch = frontmatter.author.match(/@(\w+)/);
          github_username = usernameMatch ? usernameMatch[1] : undefined;
        }

        // Get git commit hash for the feature (optional)
        // Try to find the commit that added the related files, not the doc file itself
        let commit_hash: string | undefined;
        try {
          if (frontmatter.related_files && Array.isArray(frontmatter.related_files) && frontmatter.related_files.length > 0) {
            // Use the first related file to find the implementation commit
            const relatedFile = frontmatter.related_files[0];
            // Handle paths like "flow-fcs/src/file.rs" or "flow-gates/src/gatingml.rs"
            // Remove any leading path components if they're already relative to project root
            const cleanPath = relatedFile.replace(/^(flow-fcs|flow-gates|flow-plots|peacoqc-rs|peacoqc-cli)\//, '');
            const relatedPath = join(process.cwd(), cleanPath);
            const { stdout } = await execAsync(
              `git log --diff-filter=A --format="%H" --max-count=1 -- "${relatedPath}"`,
            );
            commit_hash = stdout.trim();
            
            // If file wasn't added recently, try finding the most recent significant commit
            if (!commit_hash) {
              const { stdout: recentCommit } = await execAsync(
                `git log --format="%H" --max-count=1 --since="${frontmatter.date}" -- "${relatedPath}"`,
              );
              commit_hash = recentCommit.trim();
            }
          }
          
          // Fallback to doc file's commit if no related files
          if (!commit_hash) {
            const { stdout } = await execAsync(
              `git log -1 --format="%H" -- "${file}"`,
            );
            commit_hash = stdout.trim();
          }
        } catch {
          // Git info is optional
        }

        featureHistory.push({
          title: frontmatter.title,
          date_added: frontmatter.date,
          crate,
          author: frontmatter.author || "Unknown",
          github_username,
          excerpt,
          href,
          commit_hash,
          impact: frontmatter.impact,
          tags: frontmatter.tags,
        });
      } catch (error) {
        console.warn(`Failed to process ${file}:`, error);
      }
    }

    // Sort by date (newest first)
    featureHistory.sort(
      (a, b) =>
        new Date(b.date_added).getTime() - new Date(a.date_added).getTime(),
    );

    await writeFile(outputPath, JSON.stringify(featureHistory, null, 2));
    console.log(
      `✅ Extracted feature history for ${featureHistory.length} files to ${outputPath}`,
    );

    featureHistory.forEach((feature) => {
      console.log(
        `   - ${feature.title}: ${feature.date_added} by ${feature.author}`,
      );
    });
  } catch (error) {
    console.error("❌ Failed to extract feature history:", error);
    process.exit(1);
  }
}

async function getAllSvxFiles(dir: string): Promise<string[]> {
  const files: string[] = [];
  const entries = await readdir(dir, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = join(dir, entry.name);

    if (entry.isDirectory()) {
      const subFiles = await getAllSvxFiles(fullPath);
      files.push(...subFiles);
    } else if (entry.name === "+page.svx") {
      files.push(fullPath);
    }
  }

  return files;
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
      excerpt: content.slice(0, 200).replace(/\n/g, " ").trim(),
    };
  }

  const frontmatterText = match[1];
  const bodyText = match[2];

  // Parse YAML frontmatter using js-yaml
  let frontmatter: Frontmatter = {};
  try {
    frontmatter = yaml.load(frontmatterText) as Frontmatter;
  } catch (error) {
    console.warn("Failed to parse YAML frontmatter:", error);
    frontmatter = {};
  }

  // Extract excerpt from Summary section if available, otherwise from first paragraph
  let excerpt = "No description available.";
  
  // Try to extract from ## Summary section
  const summaryMatch = bodyText.match(/##\s+Summary\s*\n+([^#]+)/);
  if (summaryMatch) {
    excerpt = summaryMatch[1]
      .trim()
      .replace(/\n+/g, " ")
      .slice(0, 200);
  } else {
    // Fallback: find first paragraph with substantial content (not just heading)
    // Split by double newlines and find first paragraph with at least 50 chars
    const sections = bodyText.split("\n\n");
    for (const section of sections) {
      const cleaned = section
        .replace(/^#+\s+.*$/m, "") // Remove any headings
        .replace(/^\s*[-*]\s+/gm, "") // Remove list markers
        .trim();
      
      if (cleaned.length >= 50 && !cleaned.startsWith("```")) {
        excerpt = cleaned.replace(/\n/g, " ").slice(0, 200);
        break;
      }
    }
  }

  return {
    frontmatter,
    excerpt,
  };
}

// Run if called directly
if (import.meta.url === `file://${process.argv[1]}`) {
  extractFeatureHistory();
}

export { extractFeatureHistory };

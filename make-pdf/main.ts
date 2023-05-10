import puppeteer from "puppeteer";
import { opendir, writeFile, mkdir } from "fs/promises";
import path from "path";

const browser = await puppeteer.launch();
const page = await browser.newPage();

// https://gist.github.com/lovasoa/8691344
async function* walk(dir: string): AsyncGenerator<string> {
  for await (const d of await opendir(dir)) {
    const entry = path.join(dir, d.name);
    if (d.isDirectory()) yield* walk(entry);
    else if (d.isFile()) yield entry;
  }
}

await mkdir("../out/pdf", { recursive: true });

for await (const entry of await walk("../out")) {
  if (!entry.endsWith(".html")) continue;

  await page.goto(`http://localhost:8000/${entry.replaceAll("../out", "")}`);
  await page.waitForNetworkIdle();

  const outFileName = entry
    .replaceAll("../out/", "../out/pdf/")
    .replaceAll("/notes/", "/")
    .replaceAll(".html", ".pdf");
  await mkdir(path.parse(outFileName).dir, { recursive: true });
  writeFile(outFileName, await page.pdf());

  console.log(entry);
}

await browser.close();

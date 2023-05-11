import puppeteer from 'puppeteer';
import fs from 'fs/promises';
import path from 'path';

const browser = await puppeteer.launch({ headless: 'new' });
const page = await browser.newPage();

// Modified version of https://gist.github.com/lovasoa/8691344
// eslint-disable-next-line no-unused-vars
async function walk(dir: string, callback: (entry: string) => Promise<void>) {
  const dirEntry = await fs.readdir(dir, { withFileTypes: true });

  for (let i = 0; i < dirEntry.length; i++) {
    const entry = dirEntry[i];

    const filepath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      // eslint-disable-next-line no-await-in-loop
      await walk(filepath, callback);
    } else if (entry.isFile()) {
      // eslint-disable-next-line no-await-in-loop
      await callback(filepath);
    }
  }
}

await fs.mkdir('../out/pdf', { recursive: true });

await walk('../out', async (entry: string) => {
  if (!entry.endsWith('.html')) return;
  const url = `http://localhost:8000${entry.replaceAll('../out', '')}`;
  console.log(url);

  await page.goto(url);
  await page.waitForNetworkIdle();

  const outFileName = entry
    .replaceAll('../out/', '../out/pdf/')
    .replaceAll('/notes/', '/')
    .replaceAll('.html', '.pdf');
  await fs.mkdir(path.parse(outFileName).dir, { recursive: true });
  fs.writeFile(
    outFileName,
    await page.pdf({
      margin: {
        bottom: '2.5cm',
        top: '2.5cm',
        left: '0',
        right: '0',
      },
    }),
  );

  console.log(`Done: ${outFileName}`);
});

await browser.close();

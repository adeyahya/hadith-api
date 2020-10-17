const puppeteer = require("puppeteer");
const { Client } = require("pg");

const client = new Client({
  user: "postgres",
  host: "localhost",
  database: "hadith",
  password: "secretpassword",
});

const ENTRYPOINT = "https://tafsirq.com/hadits/tirmidzi";

(async () => {
  // connect database
  try {
    await client.connect();
    await client.query("SELECT NOW()");
  } catch (err) {
    console.error("error connecting to db", err);
    process.exit(1);
  }

  const browser = await puppeteer.launch();
  const page = await browser.newPage();
  await page.goto(ENTRYPOINT);

  const run = async () => {
    const hadiths_selector = ".row>.panel.panel-default";
    await page.waitForSelector(hadiths_selector);

    const hadiths = await page.$$eval(hadiths_selector, (item) => {
      return item.map((el) => {
        const number = el.querySelector(".title-header");
        const arabic = el.querySelector("p.arabic");
        const indonesian = el.querySelector("p:not(.arabic)");

        if (!number || !arabic || !indonesian) {
          return [];
        }

        return [
          9,
          number.innerText.replace(/[^\d]/g, ""),
          arabic.innerText,
          indonesian.innerText,
        ];
      });
    });

    await Promise.all(
      hadiths
        .filter((item) => item.length === 4)
        .map((values) =>
          client.query(
            "insert into public.hadiths(book_id, number, arabic, indonesian) values($1, $2, $3, $4) RETURNING *",
            values
          )
        )
    ).then((res) => {
      console.log(res.map((item) => item.rows[0]));
    });

    const hasNextPage = await page.$$eval(
      'a[rel="next"]',
      (anchor) => anchor.length > 0
    );

    if (hasNextPage) {
      await page.click('a[rel="next"]');
      await run();
    }
  };

  await run();

  await client.end();
  await browser.close();
})();

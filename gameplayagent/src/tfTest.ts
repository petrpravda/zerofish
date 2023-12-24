import { pipeline } from '@xenova/transformers';

async function run() {
  const pipe = await pipeline('sentiment-analysis');

  let response = await pipe('Everything is messed up');
  console.info(response);

  response = await pipe('Everything is cool');
  console.info(response);
}

run().then();

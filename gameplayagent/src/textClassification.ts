import { pipeline } from '@xenova/transformers';

async function run() {
  const pipe = await pipeline('text-classification');

  let response = await pipe('Dear Amazon, last week I ordered an Optimus Prime action figure\n' +
    'from your online store in Germany. Unfortunately, when I opened the package,\n' +
    'I discovered to my horror that I had been sent an action figure of Megatron\n' +
    'instead! As a lifelong enemy of the Decepticons, I hope you can understand my\n' +
    'dilemma. To resolve the issue, I demand an exchange of Megatron for the\n' +
    'Optimus Prime figure I ordered. Enclosed are copies of my records concerning\n' +
    'this purchase. I expect to hear from you soon. Sincerely, Bumblebee.');
  console.info(response);
}

run().then();

import * as readline from 'readline';
import { spawn } from 'child_process';
import { HfAgent, LLMFromHub, defaultTools } from '@huggingface/agents';

// Initialize readline interface for command line interaction
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

// Initialize Stockfish UCI interface
const stockfish = spawn('stockfish');

// Initialize Hugging Face agent for chat logic
const HF_TOKEN = process.env.HF_ACCESS_TOKEN;
const agent = new HfAgent(HF_TOKEN, LLMFromHub(HF_TOKEN), [...defaultTools]);

// Function to handle user input
rl.on('line', async (input) => {
  // Use Hugging Face agent to generate response
  const chat = [{ role: 'user', content: input }];
  // @ts-ignore
  const response = await agent.generateCode(chat);

  // Send response to Stockfish UCI interface
  stockfish.stdin.write(response + '\n');
});

// Function to handle Stockfish output
stockfish.stdout.on('data', (data) => {
  console.log(`Stockfish: ${data}`);
});

// Example client for the Solana Vanity Address Generator
// This shows how to interact with the API using JavaScript

// Function to generate a vanity address
async function generateVanityAddress(pattern, position) {
  console.log(`Generating ${position} address with pattern: ${pattern}`);
  
  // Start the generation job
  const response = await fetch('http://localhost:3001/generate', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      pattern,
      position,
    }),
  });
  
  const data = await response.json();
  if (!data.job_id) {
    throw new Error('Failed to start generation job');
  }
  
  const jobId = data.job_id;
  console.log(`Job started with ID: ${jobId}`);
  
  // Poll for results
  return pollForResults(jobId);
}

// Function to poll for job results
async function pollForResults(jobId) {
  console.log('Polling for results...');
  
  // Poll every 1 second
  while (true) {
    const response = await fetch(`http://localhost:3001/status/${jobId}`);
    const data = await response.json();
    
    if (data.status === 'complete') {
      console.log('Address found!');
      return data.result;
    } else if (data.status === 'running') {
      console.log('Still searching...');
      // Wait for 1 second before polling again
      await new Promise(resolve => setTimeout(resolve, 1000));
    } else {
      throw new Error(`Job failed or cancelled: ${JSON.stringify(data)}`);
    }
  }
}

// Example usage
async function main() {
  try {
    const result = await generateVanityAddress('abc', 'prefix');
    console.log('Generated address:');
    console.log(`Public key: ${result.public_key}`);
    console.log(`Private key: ${result.private_key}`);
  } catch (error) {
    console.error('Error:', error.message);
  }
}

main();

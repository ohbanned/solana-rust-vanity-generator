#!/usr/bin/env python3
"""
Example client for the Solana Vanity Address Generator
This shows how to interact with the API using Python
"""

import requests
import json
import time

def generate_vanity_address(pattern, position):
    """
    Generate a Solana vanity address with the given pattern and position.
    
    Args:
        pattern (str): The pattern to search for
        position (str): Either 'prefix' or 'suffix'
        
    Returns:
        dict: A dictionary containing the public_key and private_key
    """
    print(f"Generating {position} address with pattern: {pattern}")
    
    # Start the generation job
    response = requests.post(
        "http://localhost:3001/generate",
        json={"pattern": pattern, "position": position}
    )
    
    data = response.json()
    if "job_id" not in data:
        raise Exception("Failed to start generation job")
    
    job_id = data["job_id"]
    print(f"Job started with ID: {job_id}")
    
    # Poll for results
    return poll_for_results(job_id)

def poll_for_results(job_id):
    """
    Poll for the results of a job.
    
    Args:
        job_id (str): The job ID to poll for
        
    Returns:
        dict: A dictionary containing the public_key and private_key
    """
    print("Polling for results...")
    
    # Poll every 1 second
    while True:
        response = requests.get(f"http://localhost:3001/status/{job_id}")
        data = response.json()
        
        if data.get("status") == "complete":
            print("Address found!")
            return data["result"]
        elif data.get("status") == "running":
            print("Still searching...")
            # Wait for 1 second before polling again
            time.sleep(1)
        else:
            raise Exception(f"Job failed or cancelled: {json.dumps(data)}")

def main():
    """Main function to demonstrate the API usage"""
    try:
        result = generate_vanity_address("abc", "prefix")
        print("Generated address:")
        print(f"Public key: {result['public_key']}")
        print(f"Private key: {result['private_key']}")
    except Exception as e:
        print(f"Error: {str(e)}")

if __name__ == "__main__":
    main()

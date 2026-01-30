#!/usr/bin/env python3
"""
Web scraping script for Agent Skills
Reads URL and selector from environment variables
"""

import os
import sys
import json
import requests
from bs4 import BeautifulSoup

def main():
    # Get inputs from environment variables
    url = os.environ.get('INPUT_URL')
    selector = os.environ.get('INPUT_SELECTOR', 'body')
    
    if not url:
        print(json.dumps({"error": "URL is required"}))
        sys.exit(1)
    
    try:
        # Make request
        headers = {
            'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
        }
        response = requests.get(url, headers=headers, timeout=10)
        response.raise_for_status()
        
        # Parse HTML
        soup = BeautifulSoup(response.content, 'html.parser')
        
        # Extract content based on selector
        elements = soup.select(selector)
        
        if not elements:
            result = {"error": f"No elements found for selector: {selector}"}
        else:
            # Extract text from all matching elements
            texts = [elem.get_text(strip=True) for elem in elements]
            result = {
                "success": True,
                "url": url,
                "selector": selector,
                "count": len(texts),
                "data": texts
            }
        
        print(json.dumps(result, indent=2))
        
    except requests.RequestException as e:
        print(json.dumps({"error": f"Request failed: {str(e)}"}))
        sys.exit(1)
    except Exception as e:
        print(json.dumps({"error": f"Scraping failed: {str(e)}"}))
        sys.exit(1)

if __name__ == "__main__":
    main()
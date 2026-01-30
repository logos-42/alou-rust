#!/usr/bin/env python3
"""
Data analysis script for Agent Skills
Performs statistical analysis on input data
"""

import os
import sys
import json
import pandas as pd
import numpy as np

def main():
    # Get inputs from environment variables
    data_input = os.environ.get('INPUT_DATA')
    analysis_type = os.environ.get('INPUT_ANALYSIS_TYPE', 'summary')
    
    if not data_input:
        print(json.dumps({"error": "Data input is required"}))
        sys.exit(1)
    
    try:
        # Parse input data (assuming JSON format)
        if data_input.startswith('[') or data_input.startswith('{'):
            data = json.loads(data_input)
            df = pd.DataFrame(data)
        else:
            # Assume it's a file path
            if data_input.endswith('.csv'):
                df = pd.read_csv(data_input)
            elif data_input.endswith('.json'):
                df = pd.read_json(data_input)
            else:
                print(json.dumps({"error": "Unsupported file format"}))
                sys.exit(1)
        
        # Perform analysis based on type
        if analysis_type == 'summary':
            result = {
                "success": True,
                "analysis_type": "summary",
                "shape": df.shape,
                "columns": df.columns.tolist(),
                "dtypes": df.dtypes.to_dict(),
                "summary_stats": df.describe().to_dict(),
                "missing_values": df.isnull().sum().to_dict(),
                "memory_usage": df.memory_usage(deep=True).sum()
            }
        
        elif analysis_type == 'correlation':
            numeric_df = df.select_dtypes(include=[np.number])
            if not numeric_df.empty:
                corr_matrix = numeric_df.corr().to_dict()
                result = {
                    "success": True,
                    "analysis_type": "correlation",
                    "correlation_matrix": corr_matrix,
                    "strong_correlations": []
                }
                
                # Find strong correlations (> 0.7 or < -0.7)
                for col1 in corr_matrix:
                    for col2 in corr_matrix[col1]:
                        if col1 != col2 and abs(corr_matrix[col1][col2]) > 0.7:
                            result["strong_correlations"].append({
                                "var1": col1,
                                "var2": col2,
                                "correlation": corr_matrix[col1][col2]
                            })
            else:
                result = {"error": "No numeric columns found for correlation analysis"}
        
        else:
            result = {"error": f"Unknown analysis type: {analysis_type}"}
        
        print(json.dumps(result, indent=2, default=str))
        
    except Exception as e:
        print(json.dumps({"error": f"Analysis failed: {str(e)}"}))
        sys.exit(1)

if __name__ == "__main__":
    main()
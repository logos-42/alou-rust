use crate::utils::error::{AloudError, Result};
use crate::web3::contracts::Network;
use crate::web3::rpc::JsonRpcClient;
use ethabi::ethereum_types::{H160, U256};
use ethabi::{Contract, Function, Token};
use serde::Serialize;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize)]
pub struct EncodedCall {
    pub to: String,
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CallOutput {
    pub tokens: Vec<Token>,
    pub json: Value,
}

#[derive(Debug, Clone)]
pub struct ContractClient {
    address: String,
    abi: Arc<Contract>,
    rpc: JsonRpcClient,
    #[allow(dead_code)]
    pub network: Network,
}

impl ContractClient {
    pub fn new(
        address: impl Into<String>,
        abi: Arc<Contract>,
        rpc_url: String,
        network: Network,
    ) -> Self {
        Self {
            address: address.into(),
            abi,
            rpc: JsonRpcClient::new(rpc_url),
            network,
        }
    }

    #[allow(dead_code)]
    pub fn address(&self) -> &str {
        &self.address
    }

    pub async fn call(&self, function: &str, params: Vec<Token>) -> Result<CallOutput> {
        let function_ref = self.function(function)?;
        let input = function_ref.encode_input(&params)?;
        let data = format!("0x{}", hex::encode(input));

        let params = json!([
            {
                "to": self.address,
                "data": data
            },
            "latest"
        ]);

        let raw = self.rpc.call_method("eth_call", params).await?;
        let raw_str = raw
            .as_str()
            .ok_or_else(|| AloudError::RpcError("Invalid eth_call response".into()))?;

        let tokens = decode_tokens(function_ref, raw_str)?;
        let json = tokens_to_json(&tokens);

        Ok(CallOutput { tokens, json })
    }

    pub fn encode_call(
        &self,
        function: &str,
        params: Vec<Token>,
        value: Option<U256>,
    ) -> Result<EncodedCall> {
        let function_ref = self.function(function)?;
        let payload = function_ref.encode_input(&params)?;
        let data = format!("0x{}", hex::encode(payload));
        let value_hex = value.map(|v| format!("0x{:x}", v));

        Ok(EncodedCall {
            to: self.address.clone(),
            data,
            value: value_hex,
        })
    }

    fn function(&self, name: &str) -> Result<&Function> {
        self.abi.function(name).map_err(|e| {
            AloudError::InvalidInput(format!(
                "Function `{}` not found in ABI for contract {}: {}",
                name, self.address, e
            ))
        })
    }
}

pub fn parse_address(address: &str) -> Result<H160> {
    let trimmed = address.trim_start_matches("0x");
    if trimmed.len() != 40 {
        return Err(AloudError::InvalidInput(format!(
            "Invalid address length: {}",
            address
        )));
    }

    let bytes = hex::decode(trimmed)?;
    Ok(H160::from_slice(&bytes))
}

pub fn parse_u256(value: &str) -> Result<U256> {
    U256::from_dec_str(value).map_err(|e| {
        AloudError::InvalidInput(format!("Failed to parse decimal value `{}`: {}", value, e))
    })
}

#[allow(dead_code)]
pub fn parse_u256_hex(value: &str) -> Result<U256> {
    let trimmed = value.trim_start_matches("0x");
    U256::from_str_radix(trimmed, 16).map_err(|e| {
        AloudError::InvalidInput(format!("Failed to parse hex value `{}`: {}", value, e))
    })
}

pub fn token_address(address: &str) -> Result<Token> {
    Ok(Token::Address(parse_address(address)?))
}

pub fn token_uint(value: &str) -> Result<Token> {
    Ok(Token::Uint(parse_u256(value)?))
}

pub fn token_uint_u64(value: u64) -> Token {
    Token::Uint(U256::from(value))
}

#[allow(dead_code)]
pub fn token_bool(value: bool) -> Token {
    Token::Bool(value)
}

pub fn token_string(value: &str) -> Token {
    Token::String(value.to_string())
}

pub fn token_bytes_from_hex(data: &str) -> Result<Token> {
    let trimmed = data.trim_start_matches("0x");
    let bytes = hex::decode(trimmed)?;
    Ok(Token::Bytes(bytes))
}

pub fn token_fixed_bytes32(data: &str) -> Result<Token> {
    let trimmed = data.trim_start_matches("0x");
    let bytes = hex::decode(trimmed)?;
    if bytes.len() != 32 {
        return Err(AloudError::InvalidInput(format!(
            "Expected 32-byte value, got {} bytes",
            bytes.len()
        )));
    }
    Ok(Token::FixedBytes(bytes))
}

pub fn tokens_to_json(tokens: &[Token]) -> Value {
    match tokens {
        [] => Value::Null,
        [single] => token_to_json(single),
        many => Value::Array(many.iter().map(token_to_json).collect()),
    }
}

fn token_to_json(token: &Token) -> Value {
    match token {
        Token::Address(addr) => Value::String(format!("0x{:040x}", addr)),
        Token::Uint(value) => Value::String(value.to_string()),
        Token::Int(value) => Value::String(value.to_string()),
        Token::Bool(value) => Value::Bool(*value),
        Token::String(value) => Value::String(value.clone()),
        Token::FixedBytes(bytes) | Token::Bytes(bytes) => {
            Value::String(format!("0x{}", hex::encode(bytes)))
        }
        Token::Array(items) | Token::FixedArray(items) | Token::Tuple(items) => {
            Value::Array(items.iter().map(token_to_json).collect())
        }
    }
}

fn decode_tokens(function: &Function, raw: &str) -> Result<Vec<Token>> {
    let trimmed = raw.trim_start_matches("0x");
    let bytes = if trimmed.is_empty() {
        Vec::new()
    } else {
        hex::decode(trimmed)?
    };
    function
        .decode_output(&bytes)
        .map_err(|e| AloudError::InvalidInput(format!("Failed to decode output: {}", e)))
}

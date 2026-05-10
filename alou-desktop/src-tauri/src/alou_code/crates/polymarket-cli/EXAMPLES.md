# Polymarket CLI 使用示例

## 基本使用

### 1. 查看帮助
```bash
polymarket-cli --help
polymarket-cli list --help
polymarket-cli trade --help
```

### 2. 探索市场
```bash
# 列出活跃市场
polymarket-cli list --active --limit 10

# 按类别筛选
polymarket-cli list --category "politics" --active

# 查看市场详情
polymarket-cli market "will-trump-win-2024"

# 分析市场
polymarket-cli analyze "will-trump-win-2024" --timeframe "7d"
```

### 3. 钱包管理
```bash
# 连接钱包（使用私钥）
polymarket-cli wallet connect --private-key "your_private_key_here"

# 连接钱包（使用助记词）
polymarket-cli wallet connect --mnemonic "your mnemonic phrase here"

# 查看余额
polymarket-cli wallet balance

# 查看持仓
polymarket-cli wallet positions

# 查看交易历史
polymarket-cli wallet history --limit 20
```

### 4. 交易操作
```bash
# 买入订单
polymarket-cli trade "will-trump-win-2024" "YES" 100.0 --price 0.65 --side buy

# 卖出订单
polymarket-cli trade "will-trump-win-2024" "YES" 50.0 --price 0.70 --side sell

# 市价单（不指定价格）
polymarket-cli trade "will-trump-win-2024" "NO" 75.0 --side buy
```

### 5. 自动化交易
```bash
# 运行做市商机器人
polymarket-cli bot market-maker "will-trump-win-2024" --spread 0.02 --size 200

# 运行套利机器人
polymarket-cli bot arbitrage "market-id-1" "market-id-2" --threshold 0.015

# 运行趋势跟踪机器人
polymarket-cli bot trend "will-trump-win-2024" --stop-loss 0.03 --take-profit 0.08
```

### 6. 监控
```bash
# 实时监控
polymarket-cli monitor --watch --interval 30

# 单次监控
polymarket-cli monitor
```

## 高级策略

### 策略1：均值回归
1. 识别价格偏离历史均值的市场
2. 在价格过低时买入，过高时卖出
3. 设置止损和止盈

### 策略2：事件驱动
1. 监控即将发生的事件
2. 在事件前建立头寸
3. 事件发生后平仓

### 策略3：统计套利
1. 寻找相关性强的市场
2. 当价格关系偏离时进行交易
3. 等待关系回归时平仓

## 风险管理

### 资金管理
```bash
# 每次交易不超过总资金的2%
MAX_POSITION_SIZE=0.02

# 总风险敞口不超过10%
MAX_PORTFOLIO_RISK=0.10
```

### 止损设置
```bash
# 固定百分比止损
STOP_LOSS=0.05  # 5%

# 移动止损
TRAILING_STOP=0.03  # 3%
```

## 配置文件示例

创建 `.env` 文件：
```env
RPC_URL=https://polygon-rpc.com
PRIVATE_KEY=your_private_key_here
POLYMARKET_API_BASE=https://gamma-api.polymarket.com

# 交易设置
DEFAULT_SLIPPAGE=0.01
DEFAULT_GAS_LIMIT=300000

# 风险设置
MAX_POSITION_SIZE=0.02
STOP_LOSS=0.05
TAKE_PROFIT=0.10

# 机器人设置
MARKET_MAKER_SPREAD=0.01
MARKET_MAKER_SIZE=100
ARBITRAGE_THRESHOLD=0.015
```

## 故障排除

### 常见问题

1. **连接失败**
   - 检查 RPC URL
   - 验证网络连接
   - 检查防火墙设置

2. **交易失败**
   - 检查钱包余额
   - 验证 gas 设置
   - 检查合约地址

3. **API 错误**
   - 检查 API 密钥
   - 验证网络状态
   - 查看错误日志

### 调试模式
```bash
polymarket-cli --verbose trade "market-id" "YES" 100 --price 0.50 --side buy
```

## 安全建议

1. **使用硬件钱包**：对于大额资金，使用硬件钱包
2. **测试网测试**：先在测试网测试策略
3. **小额开始**：从小额交易开始
4. **定期备份**：备份钱包和配置
5. **监控日志**：定期检查交易日志

## 性能优化

1. **使用本地 RPC**：运行本地节点减少延迟
2. **批量交易**：合并小交易减少 gas 费用
3. **缓存数据**：缓存市场数据减少 API 调用
4. **异步处理**：使用异步操作提高效率
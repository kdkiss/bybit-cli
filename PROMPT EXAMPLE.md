# PROMPT EXAMPLE
Use the bybit MCP server to identify the strongest current support and resistance levels for BTCUSDT on linear, using all relevant available tools for market structure, order flow, liquidity, volatility, funding, open interest, recent trades, and any higher-timeframe context. Return the top support and resistance zones with exact prices, strength/confidence, what data supports each level, nearby invalidation/breakout signals, and clearly separate hard MCP facts from your interpretation.


# RESULTS

# BTCUSDT Linear — Support & Resistance Analysis
**Timestamp:** 2026-04-01T18:53 UTC | **Current Price:** 68,224.60

---

## 🔴 TOP RESISTANCE LEVELS

### R1: 69,301.00 — 24h High / Daily Swing High
| Attribute | Detail |
|-----------|--------|
| **Strength** | ⭐⭐⭐⭐⭐ HARD |
| **Confidence** | 95% |
| **MCP Data** | `market_tickers` `highPrice24h: 69301.00`; `market_kline D` recent daily candle high exactly 69,301; `market_kline 4H` candle #3 high 69,164.4 (failed to breach) |
| **Why it matters** | Exact 24h high rejection; confluence with daily candle high; price dropped 1,076 pts (1.56%) from this level |
| **Invalidation** | 1H close above 69,301 with volume >5,000 BTC/hr confirms breakout |
| **Breakout target** | 69,500–70,000 (psychological + prior 4H highs from 3/28–3/29) |

### R2: 68,900–69,165 — 4H Supply Zone
| Attribute | Detail |
|-----------|--------|
| **Strength** | ⭐⭐⭐⭐ STRONG |
| **Confidence** | 85% |
| **MCP Data** | `market_kline 4H` candles show repeated highs in 68,933–69,164 range over past 8h; `market_kline 1H` multiple 1H highs 68,970, 69,164, 68,858 |
| **Why it matters** | Multiple 4H candle highs rejected here; current 4H candle opened at 68,879 and sold off to 68,196 |
| **Invalidation** | 4H close above 69,165 |

### R3: 68,500–68,600 — Intraday Resistance
| Attribute | Detail |
|-----------|--------|
| **Strength** | ⭐⭐⭐ MODERATE |
| **Confidence** | 70% |
| **MCP Data** | `market_tickers` prevPrice1h: 68,408.40; multiple 1H candles bounced off this zone |
| **Why it matters** | Price failed to hold above 68,400 on recent attempt; acts as intraday pivot |
| **Invalidation** | Sustained 1H closes above 68,600 |

---

## 🟢 TOP SUPPORT LEVELS

### S1: 67,543.80 — 24h Low / Daily Swing Low
| Attribute | Detail |
|-----------|--------|
| **Strength** | ⭐⭐⭐⭐⭐ HARD |
| **Confidence** | 95% |
| **MCP Data** | `market_tickers` `lowPrice24h: 67543.80`; `market_kline D` recent daily candle low exactly 67,543.8; `market_kline 1H` candle #5 low 67,543.8 |
| **Why it matters** | Exact 24h low; daily candle low; price bounced 680 pts (1.01%) from this level |
| **Invalidation** | 1H close below 67,543 with volume >4,000 BTC/hr confirms breakdown |
| **Breakdown target** | 67,000–66,500 (prior 4H lows from 3/29–3/30) |

### S2: 67,830–67,900 — 4H Demand Zone / Recent Reaction Low
| Attribute | Detail |
|-----------|--------|
| **Strength** | ⭐⭐⭐⭐ STRONG |
| **Confidence** | 85% |
| **MCP Data** | `market_kline 4H` current 4H candle low 67,900; previous 4H candle low 67,831.7; `market_kline 1H` multiple 1H lows in 67,831–67,900 range |
| **Why it matters** | Current 4H candle wicked to 67,900 and bounced; confluence of 4H and 1H lows |
| **Invalidation** | 4H close below 67,830 |

### S3: 67,000–67,070 — Prior 4H Swing Low / Psychological Level
| Attribute | Detail |
|-----------|--------|
| **Strength** | ⭐⭐⭐ MODERATE |
| **Confidence** | 75% |
| **MCP Data** | `market_kline 4H` candle #11 low 67,067.9; `market_kline D` daily candle #2 low 65,950 (below); `market_kline 4H` candle #13 low 66,620 |
| **Why it matters** | Round psychological number; prior 4H reaction low; if 67,543 breaks, this is next major support |
| **Invalidation** | 4H close below 67,000 |

---

## 📊 ORDER FLOW & LIQUIDITY SNAPSHOT (MCP Facts)

| Metric | Value | Interpretation |
|--------|-------|----------------|
| **Spread** | 0.0001% (0.10 USDT) | Extremely tight — high liquidity |
| **Bid-side cluster** | 68,171.3 (1.85 BTC), 68,171.4 (0.59 BTC) | Immediate support ~5 pts below price |
| **Ask-side cluster** | 68,183.7 (1.25 BTC), 68,183.8 (1.44 BTC) | Immediate resistance ~41 pts above price |
| **Recent trade flow** | 90%+ buys in last 100 trades | Aggressive buying at 68,174–68,187 |
| **Large fills** | 68,174.3 (0.82 BTC total), 68,179.4 (0.32 BTC), 68,181.9 (0.56 BTC) | Institutional accumulation zones |

---

## 📈 DERIVATIVES CONTEXT (MCP Facts)

| Metric | Value | Signal |
|--------|-------|--------|
| **Funding Rate** | -0.00000532 (current), +0.00002341 (last) | Slightly negative → shorts paying longs, mild bullish |
| **Open Interest** | 50,234 BTC (rising from 48,000) | OI increasing + price declining = new short positions OR long accumulation |
| **Long/Short Ratio** | 57.89% long / 42.11% short | Consistent bullish bias (57–59% long for 20h) |
| **Index Price** | 68,261.11 | Spot trading 36 pts above perp mark — slight contango |

---

## 🎯 KEY TAKEAWAYS

**Hard MCP Facts:**
- Price is currently consolidating between **67,543.8** (24h low) and **69,301** (24h high) — a 1,757 pt range (2.57%)
- Strong buying pressure visible in recent trades (90%+ buys)
- OI rising while price consolidates suggests position building
- Funding slightly negative = shorts paying longs

**Interpretation:**
- The 67,543–67,900 zone is the strongest support cluster (daily low + 4H lows + orderflow reaction)
- The 69,165–69,301 zone is the strongest resistance cluster (daily high + 4H highs + rejection)
- Current price (68,224) sits closer to support than resistance — asymmetric risk favors longs if 67,900 holds
- Breakout above 69,301 with volume could target 70,000+; breakdown below 67,543 could target 66,500
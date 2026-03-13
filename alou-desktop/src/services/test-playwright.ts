/**
 * Playwright 浏览器服务测试脚本
 * 
 * 使用方法:
 * 1. 先确保安装了 playwright: npm install -g playwright && npx playwright install chromium
 * 2. 运行测试: npx tsx test-playwright.ts
 */

import playwrightService from './browserPlaywrightService';

async function runTests() {
  console.log('🧪 开始 Playwright 服务测试...\n');

  // 测试 1: 导航到页面并获取标题
  console.log('📋 测试 1: 导航到页面');
  try {
    const navigateResult = await playwrightService.navigate({ url: 'https://example.com' });
    console.log('  结果:', JSON.stringify(navigateResult, null, 2));
    if (navigateResult.success) {
      console.log('  ✅ 导航成功，页面标题:', navigateResult.data?.title);
    } else {
      console.log('  ❌ 导航失败:', navigateResult.error);
    }
  } catch (error) {
    console.log('  ❌ 测试出错:', error);
  }

  // 测试 2: 获取页面标题（不需要启动浏览器）
  console.log('\n📋 测试 2: 获取页面标题');
  try {
    const titleResult = await playwrightService.getPageTitle({ url: 'https://example.com' });
    console.log('  结果:', JSON.stringify(titleResult, null, 2));
    if (titleResult.success) {
      console.log('  ✅ 获取标题成功:', titleResult.data?.title);
    } else {
      console.log('  ❌ 获取标题失败:', titleResult.error);
    }
  } catch (error) {
    console.log('  ❌ 测试出错:', error);
  }

  // 测试 3: 获取页面 URL
  console.log('\n📋 测试 3: 获取页面 URL');
  try {
    const urlResult = await playwrightService.getPageUrl({ url: 'https://example.com' });
    console.log('  结果:', JSON.stringify(urlResult, null, 2));
    if (urlResult.success) {
      console.log('  ✅ 获取 URL 成功:', urlResult.data?.url);
    } else {
      console.log('  ❌ 获取 URL 失败:', urlResult.error);
    }
  } catch (error) {
    console.log('  ❌ 测试出错:', error);
  }

  // 测试 4: 获取页面源码
  console.log('\n📋 测试 4: 获取页面源码');
  try {
    const sourceResult = await playwrightService.getPageSource({ url: 'https://example.com' });
    console.log('  结果:', JSON.stringify(sourceResult, null, 2));
    if (sourceResult.success) {
      const content = sourceResult.data?.content || '';
      console.log('  ✅ 获取源码成功，内容长度:', content.length);
    } else {
      console.log('  ❌ 获取源码失败:', sourceResult.error);
    }
  } catch (error) {
    console.log('  ❌ 测试出错:', error);
  }

  // 测试 5: 执行 JavaScript
  console.log('\n📋 测试 5: 执行 JavaScript');
  try {
    const scriptResult = await playwrightService.executeScript({ 
      url: 'https://example.com',
      script: "document.title + ' - ' + window.location.href"
    });
    console.log('  结果:', JSON.stringify(scriptResult, null, 2));
    if (scriptResult.success) {
      console.log('  ✅ 执行 JS 成功:', scriptResult.data?.result);
    } else {
      console.log('  ❌ 执行 JS 失败:', scriptResult.error);
    }
  } catch (error) {
    console.log('  ❌ 测试出错:', error);
  }

  // 测试 6: 截图
  console.log('\n📋 测试 6: 截图');
  try {
    const screenshotResult = await playwrightService.screenshot({ url: 'https://example.com' });
    console.log('  结果:', screenshotResult.success ? '截图成功 (base64 长度: ' + (screenshotResult.data?.screenshot?.length || 0) + ')' : '截图失败');
    if (screenshotResult.success && screenshotResult.data?.screenshot) {
      console.log('  ✅ 截图成功，base64 长度:', screenshotResult.data.screenshot.length);
    } else {
      console.log('  ❌ 截图失败:', screenshotResult.error);
    }
  } catch (error) {
    console.log('  ❌ 测试出错:', error);
  }

  console.log('\n✨ 测试完成！');
}

runTests().catch(console.error);

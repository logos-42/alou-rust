/**
 * Playwright 浏览器自动化服务
 * 
 * 通过调用 bash 工具执行 npx playwright 来实现浏览器自动化操作
 * 跨平台兼容性好，支持 Windows/macOS/Linux
 */

import toolService from './toolService';

export interface PlaywrightResult {
  success: boolean;
  data?: any;
  output?: string;
  error?: string;
}

export interface NavigateParams {
  url: string;
}

export interface ClickParams {
  url: string;
  selector: string;
  waitTime?: number;
}

export interface InputTextParams {
  url: string;
  selector: string;
  text: string;
  clearFirst?: boolean;
}

export interface ScreenshotParams {
  url: string;
  path?: string;
}

export interface GetElementTextParams {
  url: string;
  selector: string;
}

export interface ExecuteScriptParams {
  url: string;
  script: string;
}

export interface GetPageInfoParams {
  url: string;
}

class PlaywrightService {
  /**
   * 构建 Playwright 脚本
   */
  private buildScript(action: string, params: Record<string, any>): string {
    const scripts: Record<string, string> = {
      // 导航到页面
      navigate: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          const title = await page.title();
          const url = page.url();
          console.log(JSON.stringify({ success: true, title, url }));
          await browser.close();
        })();
      `,

      // 点击元素
      click: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          ${params.waitTime ? `await page.waitForTimeout(${params.waitTime * 1000});` : ''}
          await page.click('${params.selector}');
          console.log(JSON.stringify({ success: true }));
          await browser.close();
        })();
      `,

      // 输入文本
      input_text: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          ${params.clearFirst !== false ? `await page.fill('${params.selector}', '');` : ''}
          await page.fill('${params.selector}', '${params.text.replace(/'/g, "\\'")}');
          console.log(JSON.stringify({ success: true }));
          await browser.close();
        })();
      `,

      // 截图
      screenshot: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          const screenshot = await page.screenshot({ encoding: 'base64' });
          console.log(JSON.stringify({ success: true, screenshot }));
          await browser.close();
        })();
      `,

      // 获取元素文本
      get_element_text: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          const text = await page.textContent('${params.selector}');
          console.log(JSON.stringify({ success: true, text }));
          await browser.close();
        })();
      `,

      // 执行 JavaScript
      execute_script: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          const result = await page.evaluate(() => ${params.script});
          console.log(JSON.stringify({ success: true, result: String(result) }));
          await browser.close();
        })();
      `,

      // 获取页面标题
      get_page_title: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          const title = await page.title();
          console.log(JSON.stringify({ success: true, title }));
          await browser.close();
        })();
      `,

      // 获取页面 URL
      get_page_url: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          const url = page.url();
          console.log(JSON.stringify({ success: true, url }));
          await browser.close();
        })();
      `,

      // 获取页面源码
      get_page_source: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          const content = await page.content();
          console.log(JSON.stringify({ success: true, content }));
          await browser.close();
        })();
      `,

      // 等待元素
      wait_for_element: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          await page.waitForSelector('${params.selector}', { timeout: ${(params.timeout || 10) * 1000} });
          console.log(JSON.stringify({ success: true }));
          await browser.close();
        })();
      `,

      // 滚动到元素
      scroll_to_element: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          await page.locator('${params.selector}').scrollIntoViewIfNeeded();
          console.log(JSON.stringify({ success: true }));
          await browser.close();
        })();
      `,

      // 刷新页面
      refresh: `
        const { chromium } = require('playwright');
        (async () => {
          const browser = await chromium.launch();
          const page = await browser.newPage();
          await page.goto('${params.url}');
          await page.reload();
          const title = await page.title();
          console.log(JSON.stringify({ success: true, title }));
          await browser.close();
        })();
      `
    };

    return scripts[action] || scripts.navigate;
  }

  /**
   * 执行 Playwright 操作
   * 通过调用 bash 工具来执行 npx playwright
   */
  async execute(action: string, params: Record<string, any>): Promise<PlaywrightResult> {
    const startTime = Date.now();
    
    try {
      // 构建 Playwright 脚本
      const script = this.buildScript(action, params);
      
      // 使用 bash 工具执行 npx playwright
      // 先检查 playwright 是否安装，如果没有则自动安装
      const checkAndInstallPlaywright = `
        try { require('playwright'); } catch(e) { 
          console.log('Installing playwright...'); 
          require('child_process').execSync('npm install -g playwright', { stdio: 'inherit' }); 
          require('child_process').execSync('npx playwright install chromium', { stdio: 'inherit' }); 
        }
      `;
      
      const fullScript = checkAndInstallPlaywright + ';' + script;
      
      const result = await toolService.executeTool('bash', {
        operation: 'execute',
        shell: 'node',
        command: `-e "${fullScript.replace(/"/g, '\\"').replace(/\n/g, ' ')}"`,
      });

      const executionTime = Date.now() - startTime;

      if (result.success) {
        // 解析输出
        try {
          const output = typeof result.output === 'string' 
            ? result.output.trim() 
            : JSON.stringify(result.output);
          
          // 尝试解析 JSON 输出
          let data: any = {};
          try {
            // 提取 JSON 部分（可能包含其他日志）
            const jsonMatch = output.match(/\{[\s\S]*\}/);
            if (jsonMatch) {
              data = JSON.parse(jsonMatch[0]);
            }
          } catch {
            // 如果解析失败，尝试直接使用输出
            data = { raw: output };
          }

          return {
            success: data.success !== false,
            data,
            output: `Playwright ${action} completed in ${executionTime}ms`,
          };
        } catch (parseError) {
          return {
            success: false,
            error: `Failed to parse Playwright output: ${parseError}`,
            output: result.output as string,
          };
        }
      } else {
        return {
          success: false,
          error: result.output as string || 'Playwright execution failed',
        };
      }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : String(error),
      };
    }
  }

  /**
   * 导航到指定 URL
   */
  async navigate(params: NavigateParams): Promise<PlaywrightResult> {
    return this.execute('navigate', params);
  }

  /**
   * 点击页面元素
   */
  async click(params: ClickParams): Promise<PlaywrightResult> {
    return this.execute('click', params);
  }

  /**
   * 输入文本到元素
   */
  async inputText(params: InputTextParams): Promise<PlaywrightResult> {
    return this.execute('input_text', params);
  }

  /**
   * 截取页面截图
   */
  async screenshot(params: ScreenshotParams): Promise<PlaywrightResult> {
    return this.execute('screenshot', params);
  }

  /**
   * 获取元素文本
   */
  async getElementText(params: GetElementTextParams): Promise<PlaywrightResult> {
    return this.execute('get_element_text', params);
  }

  /**
   * 执行 JavaScript 代码
   */
  async executeScript(params: ExecuteScriptParams): Promise<PlaywrightResult> {
    return this.execute('execute_script', params);
  }

  /**
   * 获取页面标题
   */
  async getPageTitle(params: GetPageInfoParams): Promise<PlaywrightResult> {
    return this.execute('get_page_title', params);
  }

  /**
   * 获取页面 URL
   */
  async getPageUrl(params: GetPageInfoParams): Promise<PlaywrightResult> {
    return this.execute('get_page_url', params);
  }

  /**
   * 获取页面源码
   */
  async getPageSource(params: GetPageInfoParams): Promise<PlaywrightResult> {
    return this.execute('get_page_source', params);
  }

  /**
   * 等待元素出现
   */
  async waitForElement(params: GetElementTextParams & { timeout?: number }): Promise<PlaywrightResult> {
    return this.execute('wait_for_element', params);
  }

  /**
   * 滚动到元素
   */
  async scrollToElement(params: GetElementTextParams): Promise<PlaywrightResult> {
    return this.execute('scroll_to_element', params);
  }

  /**
   * 刷新页面
   */
  async refresh(params: GetPageInfoParams): Promise<PlaywrightResult> {
    return this.execute('refresh', params);
  }
}

// 导出单例
const playwrightService = new PlaywrightService();
export default playwrightService;

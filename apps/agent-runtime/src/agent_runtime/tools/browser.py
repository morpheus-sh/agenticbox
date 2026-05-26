from typing import Any, Dict

from playwright.async_api import async_playwright


class BrowserTool:
    def definition(self) -> Dict[str, Any]:
        return {
            "name": "browser",
            "description": "Automate browser actions",
            "parameters": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["navigate", "screenshot", "click", "type", "extract"],
                    },
                    "url": {"type": "string"},
                    "selector": {"type": "string"},
                    "text": {"type": "string"},
                },
                "required": ["action"],
            },
        }

    async def invoke(self, args: Dict[str, Any]) -> Dict[str, Any]:
        action = args["action"]
        async with async_playwright() as p:
            browser = await p.chromium.launch(headless=True)
            ctx = await browser.new_context()
            page = await ctx.new_page()
            try:
                if action == "navigate":
                    await page.goto(args["url"])
                    return {"title": await page.title(), "url": page.url}
                elif action == "screenshot":
                    if args.get("url"):
                        await page.goto(args["url"])
                    data = await page.screenshot()
                    return {"screenshot": data.hex()}
                elif action == "click":
                    await page.click(args["selector"])
                    return {"status": "clicked"}
                elif action == "type":
                    await page.fill(args["selector"], args["text"])
                    return {"status": "typed"}
                elif action == "extract":
                    text = await page.evaluate("() => document.body.innerText")
                    return {"text": text}
            finally:
                await browser.close()

import json
from typing import Dict

from fastapi import WebSocket


class MCPServer:
    def __init__(self, tools: Dict):
        self.tools = tools

    async def handle(self, ws: WebSocket):
        while True:
            try:
                data = await ws.receive_text()
                msg = json.loads(data)
                action = msg.get("action")
                if action == "invoke":
                    result = await self.invoke(msg["tool"], msg.get("args", {}))
                    await ws.send_json({"type": "result", "id": msg.get("id"), "data": result})
                elif action == "list_tools":
                    tools = {n: t.definition() for n, t in self.tools.items()}
                    await ws.send_json({"type": "tools", "data": tools})
            except Exception as e:
                await ws.send_json({"type": "error", "message": str(e)})

    async def invoke(self, name: str, args: dict):
        tool = self.tools.get(name)
        if not tool:
            raise ValueError(f"Tool {name} not found")
        return await tool.invoke(args)

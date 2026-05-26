import httpx
from typing import Any, Dict


class HTTPTool:
    def definition(self) -> Dict[str, Any]:
        return {
            "name": "http",
            "description": "Make HTTP requests",
            "parameters": {
                "type": "object",
                "properties": {
                    "method": {"type": "string", "enum": ["GET", "POST"]},
                    "url": {"type": "string"},
                    "headers": {"type": "object"},
                    "body": {"type": "string"},
                },
                "required": ["method", "url"],
            },
        }

    async def invoke(self, args: Dict[str, Any]) -> Dict[str, Any]:
        method = args["method"]
        url = args["url"]
        headers = args.get("headers", {})
        body = args.get("body")
        async with httpx.AsyncClient(timeout=30) as client:
            if method == "GET":
                r = await client.get(url, headers=headers)
            else:
                r = await client.post(url, headers=headers, content=body)
            return {
                "status_code": r.status_code,
                "headers": dict(r.headers),
                "body": r.text,
            }

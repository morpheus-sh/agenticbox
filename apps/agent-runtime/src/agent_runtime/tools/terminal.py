import asyncio
from typing import Any, Dict


class TerminalTool:
    def definition(self) -> Dict[str, Any]:
        return {
            "name": "terminal",
            "description": "Execute shell commands",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "timeout": {"type": "integer", "default": 30},
                },
                "required": ["command"],
            },
        }

    async def invoke(self, args: Dict[str, Any]) -> Dict[str, Any]:
        cmd = args["command"]
        timeout = args.get("timeout", 30)
        proc = await asyncio.create_subprocess_shell(
            cmd, stdout=asyncio.subprocess.PIPE, stderr=asyncio.subprocess.PIPE
        )
        try:
            stdout, stderr = await asyncio.wait_for(proc.communicate(), timeout=timeout)
            return {
                "stdout": stdout.decode("utf-8", errors="replace"),
                "stderr": stderr.decode("utf-8", errors="replace"),
                "exit_code": proc.returncode,
            }
        except asyncio.TimeoutError:
            proc.kill()
            return {"error": "Command timed out"}

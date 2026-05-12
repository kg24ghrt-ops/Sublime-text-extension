import sublime
import sublime_plugin
import json
import threading
import sys
import os

# Add local 'lib' folder to path for dependencies
lib_path = os.path.join(os.path.dirname(__file__), "lib")
if lib_path not in sys.path:
    sys.path.append(lib_path)

try:
    import websockets
    import asyncio
except ImportError:
    websockets = None

class NovaCibesRunCommand(sublime_plugin.TextCommand):
    def run(self, edit):
        if not websockets:
            sublime.error_message("NovaCibes: 'websockets' library not found in the 'lib' folder.\n\nPlease ensure you included the dependency.")
            return

        # Modern VS Code-like Input Panel for the Token
        self.view.window().show_input_panel(
            "Hugging Face Token:", 
            "", 
            self.on_token_received, 
            None, None
        )

    def on_token_received(self, token):
        if not token:
            sublime.status_message("Run cancelled: Token required.")
            return
        
        # Capture current file content
        code = self.view.substr(sublime.Region(0, self.view.size()))
        # Run in a background thread to keep UI responsive
        threading.Thread(target=self.run_async, args=(token, code)).start()

    def run_async(self, token, code):
        loop = asyncio.new_event_loop()
        asyncio.set_event_loop(loop)
        loop.run_until_complete(self.execute_on_hf(token, code))

    async def execute_on_hf(self, token, code):
        window = sublime.active_window()
        # Create/Get Output Panel (the VS Code 'Terminal' feel)
        self.output_view = window.create_output_panel("novacibes_console")
        self.output_view.set_read_only(False)
        
        # Professional Styling
        settings = self.output_view.settings()
        settings.set("color_scheme", "Packages/Color Scheme - Default/Monokai.sublime-color-scheme")
        settings.set("line_numbers", False)
        settings.set("gutter", True)
        settings.set("word_wrap", True)
        
        window.run_command("show_panel", {"panel": "output.novacibes_console"})
        self.log("📡 Connecting to NovaCibes Runner...\n")

        uri = "wss://novacibes-python-running-api.hf.space/ws"
        # HF Private Spaces require Bearer token in headers for WebSocket upgrade
        headers = {"Authorization": f"Bearer {token}"}

        try:
            async with websockets.connect(uri, extra_headers=headers) as websocket:
                self.log("✅ Connected. Executing code...\n" + ("-"*40) + "\n")
                
                payload = {
                    "type": "run",
                    "session_id": f"sublime-{sublime.version()}",
                    "code": code
                }
                await websocket.send(json.dumps(payload))

                async for message in websocket:
                    data = json.loads(message)
                    if data.get("type") in ["stdout", "stderr"]:
                        self.log(data["output"])
                    elif data.get("type") == "error":
                        self.log(f"\n[SERVER ERROR]: {data['output']}\n")
                    
        except Exception as e:
            self.log(f"\n[CONNECTION FAILED]: {str(e)}\n")
            self.log("Check if your Token is correct and the Space is 'Awake'.")

    def log(self, text):
        sublime.set_timeout(lambda: self.output_view.run_command("append", {"characters": text}), 0)

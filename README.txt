# NovaCibes Sublime Text Extension

## Installation
1. Open Sublime Text.
2. Go to 'Preferences' > 'Browse Packages...'.
3. Create a folder named 'NovaCibes' (or unzip this file directly there).
4. **CRITICAL:** Because Sublime's Python is isolated, you must place the 'websockets' library inside the 'lib' folder of this plugin.
   Run this in your terminal:
   pip install websockets -t /path/to/NovaCibes/lib/

## Usage
- Press `Ctrl+Alt+R` (or use the Command Palette: 'NovaCibes: Run').
- Enter your Private Hugging Face Token when prompted.
- Results will stream in the VS Code-style bottom panel.

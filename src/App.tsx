import "./App.css";
import "@xterm/xterm/css/xterm.css";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { Terminal } from "@xterm/xterm";
import { useEffect } from "react";

export default function App() {
  useEffect(() => {
    const terminalElement = document.getElementById("terminal") as HTMLElement;
    const term = new Terminal({
      cursorBlink: true,
      fontSize: 24,
      cols: 57,
    });

    term.open(terminalElement);

    term.onKey(({ key }) => {
      console.log("Key pressed:", key);
      invoke("terminal_input", { key });
    });

    const handleOutput = (output: { payload: string }) => {
      console.log("Output received:", JSON.stringify(output.payload));

      term.write(output.payload);
    };

    listen("terminal_output", handleOutput);
  }, []);

  return <div id="terminal"></div>;
}

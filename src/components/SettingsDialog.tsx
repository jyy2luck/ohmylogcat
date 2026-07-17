import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { BUFFER_PRESETS, type BufferPreset, type Settings } from "../types";

interface SettingsDialogProps {
  open: boolean;
  onClose: () => void;
}

export default function SettingsDialog({ open, onClose }: SettingsDialogProps) {
  const [settings, setSettings] = useState<Settings>({
    bufferCapacity: 200_000,
  });
  const [selectedPreset, setSelectedPreset] = useState<BufferPreset>("Normal");
  const [customCapacity, setCustomCapacity] = useState("200000");
  const [status, setStatus] = useState<string | null>(null);

  useEffect(() => {
    if (open) {
      invoke<Settings>("load_settings")
        .then((s) => {
          setSettings(s);
          // Detect preset
          const preset = Object.entries(BUFFER_PRESETS).find(
            ([, v]) => v === s.bufferCapacity
          );
          setSelectedPreset((preset?.[0] as BufferPreset) ?? "Custom");
          setCustomCapacity(s.bufferCapacity.toString());
        })
        .catch((e) => setStatus(String(e)));
    }
  }, [open]);

  const handlePresetChange = (preset: BufferPreset) => {
    setSelectedPreset(preset);
    if (preset !== "Custom") {
      setCustomCapacity(BUFFER_PRESETS[preset].toString());
    }
  };

  const handleSave = async () => {
    const capacity =
      selectedPreset === "Custom"
        ? parseInt(customCapacity, 10) || 200_000
        : BUFFER_PRESETS[selectedPreset];

    const newSettings: Settings = {
      ...settings,
      bufferCapacity: capacity,
    };

    try {
      await invoke("save_settings", { settings: newSettings });
      setStatus("Settings saved");
      setTimeout(onClose, 800);
    } catch (e) {
      setStatus(String(e));
    }
  };

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
      <div className="bg-white rounded-lg shadow-xl w-[400px] max-h-[80vh] overflow-y-auto">
        <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200">
          <h2 className="text-base font-semibold text-gray-800">Settings</h2>
          <button
            className="text-gray-400 hover:text-gray-600 text-lg cursor-pointer"
            onClick={onClose}
          >
            ✕
          </button>
        </div>

        <div className="px-4 py-3 space-y-4">
          {/* ADB Path */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              ADB Path
            </label>
            <input
              className="w-full text-sm border border-gray-300 rounded px-2 py-1.5 bg-white placeholder:text-gray-400"
              placeholder="Leave empty to use PATH"
              value={settings.adbPath ?? ""}
              onChange={(e) =>
                setSettings((s) => ({
                  ...s,
                  adbPath: e.target.value || undefined,
                }))
              }
            />
            <p className="text-xs text-gray-400 mt-0.5">
              macOS: /opt/homebrew/bin/adb or ~/Library/Android/sdk/platform-tools/adb
            </p>
          </div>

          {/* Buffer Preset */}
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Buffer Size
            </label>
            <select
              className="w-full text-sm border border-gray-300 rounded px-2 py-1.5 bg-white mb-2"
              value={selectedPreset}
              onChange={(e) =>
                handlePresetChange(e.target.value as BufferPreset)
              }
            >
              {Object.entries(BUFFER_PRESETS).map(([key, val]) => (
                <option key={key} value={key}>
                  {key} — {val.toLocaleString()} lines
                </option>
              ))}
            </select>
            {selectedPreset === "Custom" && (
              <input
                className="w-full text-sm border border-gray-300 rounded px-2 py-1.5 bg-white"
                type="number"
                min={1000}
                max={5_000_000}
                value={customCapacity}
                onChange={(e) => setCustomCapacity(e.target.value)}
              />
            )}
            <p className="text-xs text-gray-400 mt-0.5">
              ~{(settings.bufferCapacity * 0.5 / 1024).toFixed(1)} MB estimated at capacity
            </p>
          </div>

          {status && (
            <div className="text-sm text-green-600 bg-green-50 px-2 py-1 rounded">
              {status}
            </div>
          )}
        </div>

        <div className="flex justify-end gap-2 px-4 py-3 border-t border-gray-200 bg-gray-50">
          <button
            className="px-3 py-1.5 text-sm border border-gray-300 rounded hover:bg-gray-100 cursor-pointer"
            onClick={onClose}
          >
            Cancel
          </button>
          <button
            className="px-3 py-1.5 text-sm bg-blue-600 text-white rounded hover:bg-blue-700 cursor-pointer"
            onClick={handleSave}
          >
            Save
          </button>
        </div>
      </div>
    </div>
  );
}

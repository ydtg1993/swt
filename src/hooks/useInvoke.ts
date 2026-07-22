import { invoke } from "@tauri-apps/api/core";

/** Typed wrapper around Tauri invoke. */
export async function invokeCmd<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(cmd, args);
}

// ── Settings ──────────────────────────────────────────────────
export interface AppConfig {
  asr: { default_model: string; device: string; language: string };
  tts: { default_model: string; voice: string; speed: number };
  ui: { theme: string; language: string };
  llm: { api_url: string; api_key: string; model: string; enabled: boolean; correction_prompt: string };
  export: { output_dir: string };
  api_server: { port: number; api_key: string };
  conversation: { llm_system_prompt: string; vad_mode: string; vad_energy_threshold: number };
  voice_input: { enabled: boolean; hotkey: string };
  vad: { min_silence_duration_ms: number; max_segment_time: number };
  enhance: { denoise: string; aec: string; separation: string; punctuation: string };
  advanced: { auto_save_history: boolean };
  system: { initialized: boolean };
  updater: { auto_check: boolean };
}

export async function getConfig(): Promise<AppConfig> {
  return invokeCmd<AppConfig>("get_config");
}

export async function setConfig(key: string, value: unknown): Promise<void> {
  return invokeCmd("set_config", { key, value });
}

export async function resetConfig(): Promise<AppConfig> {
  return invokeCmd<AppConfig>("reset_config");
}

// ── History ────────────────────────────────────────────────────
export interface HistoryRecord {
  id: number;
  source: string;
  file_name: string;
  file_duration: number;
  model_name: string;
  language: string;
  full_text: string;
  word_count: number;
  segments_json: string;
  created_at: string;
}

export async function getHistory(limit: number, offset: number): Promise<HistoryRecord[]> {
  return invokeCmd<HistoryRecord[]>("get_history", { limit, offset });
}

export async function searchHistory(query: string, limit: number): Promise<HistoryRecord[]> {
  return invokeCmd<HistoryRecord[]>("search_history", { query, limit });
}

export async function getHistoryCount(): Promise<number> {
  return invokeCmd<number>("get_history_count");
}

export async function deleteHistory(id: number): Promise<void> {
  return invokeCmd("delete_history", { id });
}

// ── Models ─────────────────────────────────────────────────────
export interface ModelInfo {
  id: string;
  name: string;
  description: string;
  detail_description: string;
  languages: string[];
  estimated_size: string;
  gpu_requirement: string;
  cpu_suitable: string;
  category: string;
  model_family: string;
  local_dir_name: string;
  downloaded: boolean;
  download_mode: string;
  gguf_repo_id: string;
}

export interface DownloadProgress {
  model_id: string;
  downloaded_bytes: number;
  total_bytes: number;
  percentage: number;
  state: string;
}

export async function listModels(category?: string): Promise<ModelInfo[]> {
  return invokeCmd<ModelInfo[]>("list_models", { category: category ?? null });
}

export async function startDownload(modelId: string): Promise<void> {
  return invokeCmd("start_download", { modelId });
}

export async function cancelDownload(): Promise<void> {
  return invokeCmd("cancel_download");
}

export async function deleteModelCmd(modelId: string): Promise<void> {
  return invokeCmd("delete_model", { modelId });
}

/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_API_URL?: string;
  readonly VITE_WS_URL?: string;
  readonly VITE_GRAFANA_URL?: string;
  readonly VITE_PROMETHEUS_URL?: string;
  readonly VITE_JAEGER_URL?: string;
  readonly VITE_KIBANA_URL?: string;
  readonly VITE_SMC_API_URL?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

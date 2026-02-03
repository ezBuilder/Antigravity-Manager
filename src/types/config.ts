export interface UpstreamProxyConfig {
    enabled: boolean;
    url: string;
}

export interface ProxyConfig {
    enabled: boolean;
    allow_lan_access?: boolean;
    auth_mode?: 'off' | 'strict' | 'all_except_health' | 'auto';
    port: number;
    api_key: string;
    admin_password?: string;
    auto_start: boolean;
    custom_mapping?: Record<string, string>;
    request_timeout: number;
    enable_logging: boolean;
    debug_logging?: DebugLoggingConfig;
    upstream_proxy: UpstreamProxyConfig;
    zai?: ZaiConfig;
    scheduling?: StickySessionConfig;
    experimental?: ExperimentalConfig;
    pm_router?: PmRouterConfig;
    user_agent_override?: string;
    saved_user_agent?: string;
    thinking_budget?: ThinkingBudgetConfig;
}

// ============================================================================
// Thinking Budget 配置 (控制 AI 深度思考时的 Token 预算)
// ============================================================================

/** Thinking Budget 处理模式 */
export type ThinkingBudgetMode = 'auto' | 'passthrough' | 'custom';

/** Thinking Budget 配置 */
export interface ThinkingBudgetConfig {
    /** 模式选择 */
    mode: ThinkingBudgetMode;
    /** 自定义固定值（仅在 mode=custom 时生效），范围 1024-65536 */
    custom_value: number;
}

export interface DebugLoggingConfig {
    enabled: boolean;
    output_dir?: string;
}

export type SchedulingMode = 'CacheFirst' | 'Balance' | 'PerformanceFirst';

export interface StickySessionConfig {
    mode: SchedulingMode;
    max_wait_seconds: number;
}

export type ZaiDispatchMode = 'off' | 'exclusive' | 'pooled' | 'fallback';

export interface ZaiMcpConfig {
    enabled: boolean;
    web_search_enabled: boolean;
    web_reader_enabled: boolean;
    vision_enabled: boolean;
}

export interface ZaiModelDefaults {
    opus: string;
    sonnet: string;
    haiku: string;
}

export interface ZaiConfig {
    enabled: boolean;
    base_url: string;
    api_key: string;
    dispatch_mode: ZaiDispatchMode;
    model_mapping?: Record<string, string>;
    models: ZaiModelDefaults;
    mcp: ZaiMcpConfig;
}

export interface ScheduledWarmupConfig {
    enabled: boolean;
    monitored_models: string[];
}

export interface QuotaProtectionConfig {
    enabled: boolean;
    threshold_percentage: number; // 1-99
    monitored_models: string[];
}

export interface PinnedQuotaModelsConfig {
    models: string[];
}

export interface ExperimentalConfig {
    enable_usage_scaling: boolean;
    context_compression_threshold_l1?: number;
    context_compression_threshold_l2?: number;
    context_compression_threshold_l3?: number;
}

export type PmRouterScope = 'cli_only' | 'all_requests';

export interface PmRouterConfig {
    enabled: boolean;
    scope: PmRouterScope;
    pm_lite_model: string;
    pm_pro_model: string;
    pro_keywords: string[];
    cli_user_agents: string[];
    max_context_chars: number;
    fallback_model: string;
}

export interface CircuitBreakerConfig {
    enabled: boolean;
    backoff_steps: number[];
}

export interface AppConfig {
    language: string;
    theme: string;
    auto_refresh: boolean;
    refresh_interval: number;
    auto_sync: boolean;
    sync_interval: number;
    default_export_path?: string;
    antigravity_executable?: string; // [NEW] 手动指定的反重力程序路径
    antigravity_args?: string[]; // [NEW] Antigravity 启动参数
    auto_launch?: boolean; // 开机自动启动
    auto_check_update?: boolean; // 自动检查更新
    update_check_interval?: number; // 更新检查间隔（小时）
    accounts_page_size?: number; // 账号列表每页显示数量,默认 0 表示自动计算
    scheduled_warmup: ScheduledWarmupConfig;
    quota_protection: QuotaProtectionConfig; // [NEW] 配额保护配置
    pinned_quota_models: PinnedQuotaModelsConfig; // [NEW] 配额关注列表
    circuit_breaker: CircuitBreakerConfig; // [NEW] 熔断器配置
    proxy: ProxyConfig;
}

// ============================================================================
// Cloudflared (CF隧道) 类型定义
// ============================================================================

export type TunnelMode = 'quick' | 'auth';

export interface CloudflaredConfig {
    enabled: boolean;
    mode: TunnelMode;
    port: number;
    token?: string;
    use_http2: boolean;
}

export interface CloudflaredStatus {
    installed: boolean;
    version?: string;
    running: boolean;
    url?: string;
    error?: string;
}

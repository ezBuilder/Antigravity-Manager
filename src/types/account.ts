/** Codex 사용량 (5시간 롤링 / 주간 한도) */
export interface CodexUsageInfo {
    account_id: string;
    plan_type?: string;
    primary_used_percent?: number;
    primary_window_minutes?: number;
    primary_resets_at?: number;
    secondary_used_percent?: number;
    secondary_window_minutes?: number;
    secondary_resets_at?: number;
    has_credits?: boolean;
    unlimited_credits?: boolean;
    credits_balance?: string;
    error?: string;
}

export interface Account {
    id: string;
    email: string;
    name?: string;
    provider?: string;
    token: TokenData;
    device_profile?: DeviceProfile;
    device_history?: DeviceProfileVersion[];
    quota?: QuotaData;
    /** Codex 계정 전용: 5시간/주간 사용량 (OAuth 계정만) */
    codex_usage?: CodexUsageInfo;
    disabled?: boolean;
    disabled_reason?: string;
    disabled_at?: number;
    proxy_disabled?: boolean;
    proxy_disabled_reason?: string;
    proxy_disabled_at?: number;
    protected_models?: string[];
    created_at: number;
    last_used: number;
}

export interface TokenData {
    access_token: string;
    refresh_token: string;
    expires_in: number;
    expiry_timestamp: number;
    token_type: string;
    email?: string;
}

export interface QuotaData {
    models: ModelQuota[];
    last_updated: number;
    is_forbidden?: boolean;
    subscription_tier?: string;  // 订阅类型: FREE/PRO/ULTRA
}

export interface ModelQuota {
    name: string;
    percentage: number;
    reset_time: string;
}

export interface DeviceProfile {
    machine_id: string;
    mac_machine_id: string;
    dev_device_id: string;
    sqm_id: string;
}

export interface DeviceProfileVersion {
    id: string;
    created_at: number;
    label: string;
    profile: DeviceProfile;
    is_current?: boolean;
}

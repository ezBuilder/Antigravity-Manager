import { useMemo } from 'react';
import { ArrowRightLeft, RefreshCw, Trash2, Download, Info, Lock, Ban, Diamond, Gem, Circle, ToggleLeft, ToggleRight, Fingerprint, Sparkles } from 'lucide-react';
import { Account } from '../../types/account';
import { cn } from '../../utils/cn';
import { useTranslation } from 'react-i18next';
import { useConfigStore } from '../../stores/useConfigStore';
import { QuotaItem } from './QuotaItem';

interface AccountCardProps {
    account: Account;
    selected: boolean;
    onSelect: () => void;
    isCurrent: boolean;
    isRefreshing: boolean;
    isSwitching?: boolean;
    onSwitch: () => void;
    onRefresh: () => void;
    onViewDevice: () => void;
    onViewDetails: () => void;
    onExport: () => void;
    onDelete: () => void;
    onToggleProxy: () => void;
    onWarmup?: () => void;
}

const DEFAULT_MODELS = [
    { id: 'gemini-3-pro-high', label: 'G3 Pro', protectedKey: 'gemini-pro' },
    { id: 'gemini-3-flash', label: 'G3 Flash', protectedKey: 'gemini-flash' },
    { id: 'gemini-3-pro-image', label: 'G3 Image', protectedKey: 'gemini-image' },
    { id: 'claude-sonnet-4-5-thinking', label: 'Claude 4.5', protectedKey: 'claude' }
];

function AccountCard({ account, selected, onSelect, isCurrent: propIsCurrent, isRefreshing, isSwitching = false, onSwitch, onRefresh, onViewDetails, onExport, onDelete, onToggleProxy, onViewDevice, onWarmup }: AccountCardProps) {
    const { t } = useTranslation();
    const { config, showAllQuotas } = useConfigStore();
    const isDisabled = Boolean(account.disabled);
    const isCodex = account.provider === 'Codex';

    // Use the prop directly from parent component
    const isCurrent = propIsCurrent;

    const displayModels = useMemo(() => {
        // Build map of friendly labels from DEFAULT_MODELS
        const labelMap = new Map(DEFAULT_MODELS.map(m => [m.id, m.label]));

        // Get all models from account (source of truth)
        const accountModels = account.quota?.models?.map(m => ({
            id: m.name,
            label: labelMap.get(m.name) || m.name,
            protectedKey: DEFAULT_MODELS.find(d => d.id === m.name)?.protectedKey,
            data: m
        })) || [];

        if (showAllQuotas) return accountModels;

        // Filter for pinned or defaults
        const pinned = config?.pinned_quota_models?.models;
        if (pinned && pinned.length > 0) {
            return accountModels.filter(m => pinned.includes(m.id));
        }

        // Default fallback: show known default models
        return accountModels.filter(m => DEFAULT_MODELS.some(d => d.id === m.id));
    }, [config, account, showAllQuotas]);

    const isModelProtected = (key?: string) => {
        if (!key) return false;
        return account.protected_models?.includes(key);
    };

    return (
        <div className={cn(
            "flex flex-col p-3 rounded-xl border transition-all hover:shadow-md",
            isCurrent
                ? "bg-blue-50/30 border-blue-200 dark:bg-blue-900/10 dark:border-blue-900/30"
                : "bg-white dark:bg-base-100 border-gray-200 dark:border-base-300",
            (isRefreshing || isDisabled) && "opacity-70"
        )}>

            {/* Header: Checkbox + Email + Badges */}
            <div className="flex-none flex items-start gap-3 mb-2">
                <input
                    type="checkbox"
                    className="mt-1 checkbox checkbox-xs rounded border-2 border-gray-400 dark:border-gray-500 checked:border-blue-600 checked:bg-blue-600 [--chkbg:theme(colors.blue.600)] [--chkfg:white]"
                    checked={selected}
                    onChange={() => onSelect()}
                    onClick={(e) => e.stopPropagation()}
                />
                <div className="flex-1 min-w-0 flex flex-col gap-1.5">
                    <h3 className={cn(
                        "font-semibold text-sm truncate w-full",
                        isCurrent ? "text-blue-700 dark:text-blue-400" : "text-gray-900 dark:text-base-content"
                    )} title={account.email}>
                        {account.email}
                    </h3>
                    <div className="flex items-center justify-between w-full gap-2">
                        <div className="flex items-center gap-1.5 flex-wrap">
                            {isCurrent && (
                                <span className="px-1.5 py-0.5 rounded-md bg-blue-100 dark:bg-blue-900/40 text-blue-700 dark:text-blue-300 text-[9px] font-bold shadow-sm border border-blue-200/50">
                                    {t('accounts.current').toUpperCase()}
                                </span>
                            )}
                            {isDisabled && (
                                <span
                                    className="px-1.5 py-0.5 rounded-md bg-rose-100 dark:bg-rose-900/40 text-rose-700 dark:text-rose-300 text-[9px] font-bold flex items-center gap-1 shadow-sm border border-rose-200/50"
                                    title={account.disabled_reason || t('accounts.disabled_tooltip')}
                                >
                                    <Ban className="w-2.5 h-2.5" />
                                    {t('accounts.disabled').toUpperCase()}
                                </span>
                            )}
                            {account.proxy_disabled && (
                                <span
                                    className="px-1.5 py-0.5 rounded-md bg-orange-100 dark:bg-orange-900/40 text-orange-700 dark:text-orange-300 text-[9px] font-bold flex items-center gap-1 shadow-sm border border-orange-200/50"
                                    title={account.proxy_disabled_reason || t('accounts.proxy_disabled_tooltip')}
                                >
                                    <Ban className="w-2.5 h-2.5" />
                                    {t('accounts.proxy_disabled').toUpperCase()}
                                </span>
                            )}
                            {isCodex && (
                                <>
                                    <span className="px-1.5 py-0.5 rounded-md bg-violet-100 dark:bg-violet-900/40 text-violet-700 dark:text-violet-300 text-[9px] font-bold shadow-sm border border-violet-200/50">
                                        CODEX
                                    </span>
                                    {account.codex_usage?.plan_type && (() => {
                                        const planType = account.codex_usage.plan_type.toLowerCase();
                                        if (planType.includes('plus')) {
                                            return (
                                                <span className="flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-gradient-to-r from-green-600 to-emerald-600 text-white text-[9px] font-bold shadow-sm">
                                                    <Diamond className="w-2.5 h-2.5 fill-current" />
                                                    PLUS
                                                </span>
                                            );
                                        } else if (planType.includes('pro')) {
                                            return (
                                                <span className="flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-[9px] font-bold shadow-sm">
                                                    <Diamond className="w-2.5 h-2.5 fill-current" />
                                                    PRO
                                                </span>
                                            );
                                        } else if (planType.includes('team')) {
                                            return (
                                                <span className="flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-gradient-to-r from-purple-600 to-pink-600 text-white text-[9px] font-bold shadow-sm">
                                                    <Gem className="w-2.5 h-2.5 fill-current" />
                                                    TEAM
                                                </span>
                                            );
                                        } else {
                                            return (
                                                <span className="flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-gray-100 dark:bg-white/10 text-gray-500 dark:text-gray-400 text-[9px] font-bold shadow-sm border border-gray-200 dark:border-white/10">
                                                    <Circle className="w-2.5 h-2.5" />
                                                    FREE
                                                </span>
                                            );
                                        }
                                    })()}
                                </>
                            )}
                            {account.quota?.is_forbidden && (
                                <span className="px-1.5 py-0.5 rounded-md bg-red-100 dark:bg-red-900/40 text-red-600 dark:text-red-400 text-[9px] font-bold flex items-center gap-1 shadow-sm border border-red-200/50" title={t('accounts.forbidden_tooltip')}>
                                    <Lock className="w-2.5 h-2.5" />
                                    {t('accounts.forbidden').toUpperCase()}
                                </span>
                            )}
                            {/* 订阅类型徽章 */}
                            {account.quota?.subscription_tier && (() => {
                                const tier = account.quota.subscription_tier.toLowerCase();
                                if (tier.includes('ultra')) {
                                    return (
                                        <span className="flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-gradient-to-r from-purple-600 to-pink-600 text-white text-[9px] font-bold shadow-sm">
                                            <Gem className="w-2.5 h-2.5 fill-current" />
                                            ULTRA
                                        </span>
                                    );
                                } else if (tier.includes('pro')) {
                                    return (
                                        <span className="flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-gradient-to-r from-blue-600 to-indigo-600 text-white text-[9px] font-bold shadow-sm">
                                            <Diamond className="w-2.5 h-2.5 fill-current" />
                                            PRO
                                        </span>
                                    );
                                } else {
                                    return (
                                        <span className="flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-gray-100 dark:bg-white/10 text-gray-500 dark:text-gray-400 text-[9px] font-bold shadow-sm border border-gray-200 dark:border-white/10">
                                            <Circle className="w-2.5 h-2.5" />
                                            FREE
                                        </span>
                                    );
                                }
                            })()}
                        </div>
                        <span className="text-[10px] text-gray-400 dark:text-gray-500 font-mono shrink-0 whitespace-nowrap">
                            {new Date(account.last_used * 1000).toLocaleString([], { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' })}
                        </span>
                    </div>
                </div>
            </div>

            {/* 配额展示 / Codex 사용량 */}
            <div className="flex-1 px-2 mb-2 overflow-y-auto scrollbar-none">
                {isCodex ? (
                    account.codex_usage?.error ? (
                        <div className="flex items-center gap-2 text-xs text-amber-600 dark:text-amber-400 bg-amber-50/50 dark:bg-amber-900/10 p-2 rounded-lg border border-amber-100 dark:border-amber-900/30">
                            <span>{account.codex_usage.error}</span>
                        </div>
                    ) : account.codex_usage ? (
                        <div className="grid grid-cols-1 gap-2 content-start">
                            {account.codex_usage.primary_used_percent != null && (
                                <QuotaItem
                                    key="codex-5h"
                                    label={t('accounts.codex_usage_5h', '5h')}
                                    percentage={100 - Math.round(account.codex_usage.primary_used_percent)}
                                    resetTime={
                                        account.codex_usage.primary_resets_at != null
                                            ? new Date(account.codex_usage.primary_resets_at * 1000).toISOString()
                                            : undefined
                                    }
                                />
                            )}
                            {account.codex_usage.secondary_used_percent != null && (
                                <QuotaItem
                                    key="codex-weekly"
                                    label={t('accounts.codex_usage_weekly', 'Weekly')}
                                    percentage={100 - Math.round(account.codex_usage.secondary_used_percent)}
                                    resetTime={
                                        account.codex_usage.secondary_resets_at != null
                                            ? new Date(account.codex_usage.secondary_resets_at * 1000).toISOString()
                                            : undefined
                                    }
                                />
                            )}
                            {account.codex_usage.plan_type && (
                                <div className="text-[10px] text-gray-400 dark:text-gray-500 font-mono px-1.5">
                                    {account.codex_usage.plan_type}
                                </div>
                            )}
                            {account.codex_usage.primary_used_percent == null && account.codex_usage.secondary_used_percent == null && !account.codex_usage.error && (
                                <div className="text-[10px] text-gray-400 dark:text-gray-500 italic px-1.5">
                                    {t('accounts.codex_usage_loading', 'Loading usage…')}
                                </div>
                            )}
                        </div>
                    ) : (
                        <div className="text-[10px] text-gray-400 dark:text-gray-500 italic px-1.5">
                            {t('accounts.codex_usage_loading', 'Loading usage…')}
                        </div>
                    )
                ) : account.quota?.is_forbidden ? (
                    <div className="flex items-center gap-2 text-xs text-red-500 dark:text-red-400 bg-red-50/50 dark:bg-red-900/10 p-2 rounded-lg border border-red-100 dark:border-red-900/30">
                        <Ban className="w-4 h-4 shrink-0" />
                        <span>{t('accounts.forbidden_msg')}</span>
                    </div>
                ) : (
                    <div className="grid grid-cols-1 gap-2 content-start">
                        {displayModels.map((model) => (
                            <QuotaItem
                                key={model.id}
                                label={model.label}
                                percentage={model.data?.percentage || 0}
                                resetTime={model.data?.reset_time}
                                isProtected={isModelProtected(model.protectedKey)}
                            />
                        ))}
                    </div>
                )}
            </div>

            {/* Footer: Actions Only */}
            <div className="flex-none flex items-center justify-center pt-2 pb-1 border-t border-gray-100 dark:border-base-200">
                <div className="flex flex-wrap items-center justify-center gap-1 w-full">
                    <button
                        className="p-1.5 text-gray-400 hover:text-sky-600 dark:hover:text-sky-400 hover:bg-sky-50 dark:hover:bg-sky-900/30 rounded-lg transition-all"
                        onClick={(e) => { e.stopPropagation(); onViewDetails(); }}
                        title={t('common.details')}
                    >
                        <Info className="w-3.5 h-3.5" />
                    </button>
                    {!isCodex && (
                        <button
                            className="p-1.5 text-gray-400 hover:text-indigo-600 dark:hover:text-indigo-400 hover:bg-indigo-50 dark:hover:bg-indigo-900/30 rounded-lg transition-all"
                            onClick={(e) => { e.stopPropagation(); onViewDevice(); }}
                            title={t('accounts.device_fingerprint')}
                        >
                            <Fingerprint className="w-3.5 h-3.5" />
                        </button>
                    )}
                    {!isCodex && (
                        <button
                            className={`p-1.5 rounded-lg transition-all ${(isSwitching || isDisabled) ? 'text-blue-600 bg-blue-50 dark:text-blue-400 dark:bg-blue-900/10 cursor-not-allowed' : 'text-gray-400 hover:text-blue-600 dark:hover:text-blue-400 hover:bg-blue-50 dark:hover:bg-blue-900/30'}`}
                            onClick={(e) => { e.stopPropagation(); onSwitch(); }}
                            title={isDisabled ? t('accounts.disabled_tooltip') : (isSwitching ? t('common.loading') : t('common.switch'))}
                            disabled={isSwitching || isDisabled}
                        >
                            <ArrowRightLeft className={`w-3.5 h-3.5 ${isSwitching ? 'animate-spin' : ''}`} />
                        </button>
                    )}
                    {!isCodex && onWarmup && (
                        <button
                            className={`p-1.5 rounded-lg transition-all ${(isRefreshing || isDisabled) ? 'text-orange-600 bg-orange-50 dark:bg-orange-900/10 cursor-not-allowed' : 'text-gray-400 hover:text-orange-500 hover:bg-orange-50 dark:hover:bg-orange-900/30'}`}
                            onClick={(e) => { e.stopPropagation(); onWarmup(); }}
                            title={isDisabled ? t('accounts.disabled_tooltip') : (isRefreshing ? t('common.loading') : t('accounts.warmup_this', '预热该账号'))}
                            disabled={isRefreshing || isDisabled}
                        >
                            <Sparkles className={`w-3.5 h-3.5 ${isRefreshing ? 'animate-pulse' : ''}`} />
                        </button>
                    )}
                    <button
                        className={`p-1.5 rounded-lg transition-all ${isRefreshing
                            ? 'text-green-600 bg-green-50 dark:bg-green-900/10'
                            : 'text-gray-400 hover:text-green-600 dark:hover:text-green-400 hover:bg-green-50 dark:hover:bg-green-900/30'}`}
                        onClick={(e) => { e.stopPropagation(); onRefresh(); }}
                        disabled={isRefreshing || isDisabled}
                        title={isDisabled ? t('accounts.disabled_tooltip') : t('common.refresh')}
                    >
                        <RefreshCw className={`w-3.5 h-3.5 ${isRefreshing ? 'animate-spin' : ''}`} />
                    </button>
                    {!isCodex && (
                        <button
                            className="p-1.5 text-gray-400 hover:text-indigo-600 hover:bg-indigo-50 rounded-lg transition-all"
                            onClick={(e) => { e.stopPropagation(); onExport(); }}
                            title={t('common.export')}
                        >
                            <Download className="w-3.5 h-3.5" />
                        </button>
                    )}
                    {!isCodex && (
                        <button
                            className={cn(
                                "p-1.5 rounded-lg transition-all",
                                account.proxy_disabled
                                    ? "text-gray-400 hover:text-green-600 hover:bg-green-50"
                                    : "text-gray-400 hover:text-orange-600 hover:bg-orange-50"
                            )}
                            onClick={(e) => { e.stopPropagation(); onToggleProxy(); }}
                            title={account.proxy_disabled ? t('accounts.enable_proxy') : t('accounts.disable_proxy')}
                        >
                            {account.proxy_disabled ? (
                                <ToggleRight className="w-3.5 h-3.5" />
                            ) : (
                                <ToggleLeft className="w-3.5 h-3.5" />
                            )}
                        </button>
                    )}
                    <button
                        className="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-all"
                        onClick={(e) => { e.stopPropagation(); onDelete(); }}
                        title={t('common.delete')}
                    >
                        <Trash2 className="w-3.5 h-3.5" />
                    </button>
                </div>
            </div>
        </div >
    );
}

export default AccountCard;

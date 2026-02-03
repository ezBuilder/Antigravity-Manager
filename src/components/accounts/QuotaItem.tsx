
import { Clock, Lock } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '../../utils/cn';
import { getQuotaColor, formatTimeRemaining, getTimeRemainingColor } from '../../utils/format';

interface QuotaItemProps {
    label: string;
    percentage: number;
    resetTime?: string;
    isProtected?: boolean;
    /** Codex: percentage = 사용량(used). remaining이면 높을수록 좋음, used면 낮을수록 좋음 */
    variant?: 'remaining' | 'used';
    className?: string;
}

/** 사용량(used)일 때: 낮을수록 여유 → success, 높을수록 한도 근접 → error */
function getUsedPercentColor(p: number): string {
    if (p < 20) return 'success';
    if (p < 50) return 'warning';
    return 'error';
}

export function QuotaItem({ label, percentage, resetTime, isProtected, variant = 'remaining', className }: QuotaItemProps) {
    const { t } = useTranslation();
    const color = variant === 'used' ? getUsedPercentColor(percentage) : getQuotaColor(percentage);
    const getBgColorClass = (c: string) => {
        switch (c) {
            case 'success': return 'bg-emerald-500';
            case 'warning': return 'bg-amber-500';
            case 'error': return 'bg-rose-500';
            default: return 'bg-gray-500';
        }
    };
    const getTextColorClass = (c: string) => {
        switch (c) {
            case 'success': return 'text-emerald-600 dark:text-emerald-400';
            case 'warning': return 'text-amber-600 dark:text-amber-400';
            case 'error': return 'text-rose-600 dark:text-rose-400';
            default: return 'text-gray-500';
        }
    };

    const getTimeColorClass = (time?: string) => {
        if (!time) return 'text-gray-300 dark:text-gray-600';
        const color = getTimeRemainingColor(time);
        switch (color) {
            case 'success': return 'text-emerald-600 dark:text-emerald-400';
            case 'warning': return 'text-amber-600 dark:text-amber-400';
            default: return 'text-gray-400 dark:text-gray-500 opacity-60';
        }
    };

    return (
        <div className={cn(
            "relative h-[22px] flex items-center px-1.5 rounded-md overflow-hidden border border-gray-100/50 dark:border-white/5 bg-gray-50/30 dark:bg-white/5 group/quota",
            className
        )}>
            {/* Background Progress Bar */}
            <div
                className={cn(
                    "absolute inset-y-0 left-0 transition-all duration-700 ease-out opacity-15 dark:opacity-20",
                    getBgColorClass(color)
                )}
                style={{ width: `${percentage}%` }}
            />

            {/* Content */}
            <div className="relative z-10 w-full flex items-center text-[10px] font-mono leading-none gap-1.5">
                {/* Model Name */}
                <span className="flex-1 min-w-0 text-gray-500 dark:text-gray-400 font-bold truncate text-left" title={label}>
                    {label}
                </span>

                {/* Reset Time */}
                <div className="w-[58px] flex justify-start shrink-0">
                    {resetTime ? (
                        <span className={cn("flex items-center gap-0.5 font-medium transition-colors truncate", getTimeColorClass(resetTime))}>
                            <Clock className="w-2.5 h-2.5 shrink-0" />
                            {formatTimeRemaining(resetTime)}
                        </span>
                    ) : (
                        <span className="text-gray-300 dark:text-gray-600 italic scale-90">N/A</span>
                    )}
                </div>

                {/* Percentage (remaining = 남은 할당, used = 사용량) */}
                <span className={cn("text-right font-bold transition-colors flex items-center justify-end gap-0.5 shrink-0 min-w-[52px]", getTextColorClass(color))} title={variant === 'used' ? t('accounts.codex_used_tooltip', 'Used in this window') : undefined}>
                    {isProtected && (
                        <span title={t('accounts.quota_protected')}><Lock className="w-2.5 h-2.5 text-amber-500" /></span>
                    )}
                    {percentage}%{variant === 'used' ? ` ${t('accounts.codex_used_suffix', 'used')}` : ''}
                </span>
            </div>
        </div>
    );
}

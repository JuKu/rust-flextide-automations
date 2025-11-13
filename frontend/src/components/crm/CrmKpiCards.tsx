"use client";

import { CrmKpiResponse } from "@/lib/api";

interface CrmKpiCardsProps {
  kpis: CrmKpiResponse;
}

export function CrmKpiCards({ kpis }: CrmKpiCardsProps) {
  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat("de-DE", {
      style: "currency",
      currency: "EUR",
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(value);
  };

  const formatPercentage = (value: number) => {
    return `${value.toFixed(1)}%`;
  };

  const getOrdersComparison = () => {
    if (kpis.orders_last_month === 0) {
      return { text: "—", color: "text-flextide-neutral-text-medium" };
    }
    const diff = kpis.orders_this_month - kpis.orders_last_month;
    const percentChange = ((diff / kpis.orders_last_month) * 100).toFixed(1);
    const isPositive = diff >= 0;
    return {
      text: `${isPositive ? "+" : ""}${percentChange}%`,
      color: isPositive
        ? "text-flextide-success"
        : "text-flextide-error",
    };
  };

  const ordersComparison = getOrdersComparison();

  return (
    <div className="grid grid-cols-1 gap-4 md:grid-cols-2 lg:grid-cols-3 mb-6">
      {/* Total Sales this Month */}
      <div className="rounded-lg bg-flextide-neutral-panel-bg border-2 border-flextide-primary-accent p-6 shadow-sm">
        <div className="text-sm text-flextide-neutral-text-medium mb-1">
          Total Sales this Month
        </div>
        <div className="text-2xl font-semibold text-flextide-neutral-text-dark">
          {formatCurrency(kpis.total_sales_this_month)}
        </div>
      </div>

      {/* Orders this Month */}
      <div className="rounded-lg bg-flextide-neutral-panel-bg border-2 border-flextide-secondary-teal p-6 shadow-sm">
        <div className="text-sm text-flextide-neutral-text-medium mb-1">
          Orders this Month
        </div>
        <div className="flex items-baseline gap-2">
          <div className="text-2xl font-semibold text-flextide-neutral-text-dark">
            {kpis.orders_this_month}
          </div>
          <div className={`text-sm font-medium ${ordersComparison.color}`}>
            {ordersComparison.text}
          </div>
        </div>
        <div className="text-xs text-flextide-neutral-text-medium mt-1">
          vs. last month
        </div>
      </div>

      {/* Win Rate this Month */}
      <div className="rounded-lg bg-flextide-neutral-panel-bg border-2 border-flextide-success p-6 shadow-sm">
        <div className="text-sm text-flextide-neutral-text-medium mb-1">
          Win Rate this Month
        </div>
        <div className="text-2xl font-semibold text-flextide-neutral-text-dark">
          {formatPercentage(kpis.win_rate_this_month)}
        </div>
      </div>

      {/* Avg. days to Close */}
      <div className="rounded-lg bg-flextide-neutral-panel-bg border-2 border-flextide-warning p-6 shadow-sm">
        <div className="text-sm text-flextide-neutral-text-medium mb-1">
          Avg. days to Close
        </div>
        <div className="text-2xl font-semibold text-flextide-neutral-text-dark">
          {kpis.avg_days_to_close.toFixed(1)}
        </div>
      </div>

      {/* Total customers in CRM system */}
      <div className="rounded-lg bg-flextide-neutral-panel-bg border-2 border-flextide-secondary-purple p-6 shadow-sm">
        <div className="text-sm text-flextide-neutral-text-medium mb-1">
          Total customers in CRM system
        </div>
        <div className="text-2xl font-semibold text-flextide-neutral-text-dark">
          {kpis.total_users}
        </div>
      </div>

      {/* Open Deals */}
      <div className="rounded-lg bg-flextide-neutral-panel-bg border-2 border-flextide-info p-6 shadow-sm">
        <div className="text-sm text-flextide-neutral-text-medium mb-1">
          Open Deals
        </div>
        <div className="text-2xl font-semibold text-flextide-neutral-text-dark">
          {formatCurrency(kpis.open_deals)}
        </div>
      </div>
    </div>
  );
}


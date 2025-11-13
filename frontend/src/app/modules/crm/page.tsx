"use client";

import { useState, useEffect } from "react";
import { AppLayout } from "@/components/layout/AppLayout";
import { CrmKpiCards } from "@/components/crm/CrmKpiCards";
import { CrmCustomersSection } from "@/components/crm/CrmCustomersSection";
import { PieChart } from "@/components/common/PieChart";
import { LineChart } from "@/components/common/LineChart";
import {
  getCrmKpis,
  getCrmCustomers,
  getCrmSalesPipelineChart,
  getCrmCountriesChart,
  getCrmClosedDeals,
  type CrmKpiResponse,
  type CrmCustomer,
  type CrmPipelineStatus,
  type CrmCountryData,
  type CrmClosedDealData,
} from "@/lib/api";

export default function CrmPage() {
  const [kpis, setKpis] = useState<CrmKpiResponse | null>(null);
  const [customers, setCustomers] = useState<CrmCustomer[]>([]);
  const [pipelineData, setPipelineData] = useState<CrmPipelineStatus[]>([]);
  const [countriesData, setCountriesData] = useState<CrmCountryData[]>([]);
  const [closedDealsData, setClosedDealsData] = useState<CrmClosedDealData[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchData() {
      try {
        setLoading(true);
        setError(null);

        const [kpisData, customersData, pipelineData, countriesData, closedDealsData] =
          await Promise.all([
            getCrmKpis(),
            getCrmCustomers(),
            getCrmSalesPipelineChart(),
            getCrmCountriesChart(),
            getCrmClosedDeals(),
          ]);

        setKpis(kpisData);
        setCustomers(customersData.customers);
        setPipelineData(pipelineData.statuses);
        setCountriesData(countriesData.countries);
        setClosedDealsData(closedDealsData.deals);
      } catch (err) {
        console.error("Failed to fetch CRM data:", err);
        setError(err instanceof Error ? err.message : "Failed to load CRM data");
      } finally {
        setLoading(false);
      }
    }

    fetchData();
  }, []);

  const handleCreateCustomer = () => {
    // TODO: Implement customer creation
    console.log("Create customer");
  };

  if (loading) {
    return (
      <AppLayout>
        <div className="mx-auto max-w-7xl px-6 py-8">
          <div className="flex items-center justify-center h-64">
            <div className="text-flextide-neutral-text-medium">Loading...</div>
          </div>
        </div>
      </AppLayout>
    );
  }

  if (error) {
    return (
      <AppLayout>
        <div className="mx-auto max-w-7xl px-6 py-8">
          <div className="rounded-lg bg-flextide-error/10 border border-flextide-error p-4 text-flextide-error">
            Error: {error}
          </div>
        </div>
      </AppLayout>
    );
  }

  if (!kpis) {
    return null;
  }

  // Transform pipeline data for PieChart
  const pipelineChartData = pipelineData.map((item) => ({
    label: item.status,
    value: item.count,
  }));

  // Transform countries data for PieChart
  const countriesChartData = countriesData.map((item) => ({
    label: item.country,
    value: item.count,
  }));

  // Transform closed deals data for LineChart
  const closedDealsSeries = [
    {
      label: "Current Year",
      data: closedDealsData.map((deal) => ({
        label: deal.month,
        value: deal.current_year,
      })),
      color: "#5667FF",
    },
    {
      label: "Previous Year",
      data: closedDealsData.map((deal) => ({
        label: deal.month,
        value: deal.previous_year,
      })),
      color: "#9E9E9E",
    },
  ];

  return (
    <AppLayout>
      <div className="mx-auto max-w-7xl px-6 py-8">
        <div className="mb-8">
          <h1 className="text-3xl font-semibold text-flextide-neutral-text-dark mb-2">
            CRM
          </h1>
          <p className="text-flextide-neutral-text-medium">
            Manage your customer relationships and sales pipeline
          </p>
        </div>

        {/* First Row: KPI Cards */}
        <CrmKpiCards kpis={kpis} />

        {/* Second Row: Customers Section */}
        <div className="mb-6">
          <CrmCustomersSection
            customers={customers}
            onCreateCustomer={handleCreateCustomer}
          />
        </div>

        {/* Third Row: Charts (3 columns) */}
        <div className="grid grid-cols-1 gap-6 lg:grid-cols-3">
          {/* Column 1: Sales Pipeline Chart */}
          <div>
            <PieChart
              title="Sales Pipeline Status"
              data={pipelineChartData}
            />
          </div>

          {/* Column 2: Countries Chart */}
          <div>
            <PieChart
              title="Customers by Country"
              data={countriesChartData}
            />
          </div>

          {/* Column 3: Closed Deals Chart */}
          <div>
            <LineChart
              title="Closed Deals (Last 12 Months)"
              series={closedDealsSeries}
              yAxisLabel="Deals"
              height={300}
            />
          </div>
        </div>
      </div>
    </AppLayout>
  );
}


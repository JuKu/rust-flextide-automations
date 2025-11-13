"use client";

import { useState, useEffect } from "react";
import { AppLayout } from "@/components/layout/AppLayout";
import { CrmKpiCards } from "@/components/crm/CrmKpiCards";
import { CrmCustomersSection } from "@/components/crm/CrmCustomersSection";
import { CreateCustomerDialog } from "@/components/crm/CreateCustomerDialog";
import { PieChart } from "@/components/common/PieChart";
import { LineChart } from "@/components/common/LineChart";
import { getCurrentOrganizationUuid } from "@/lib/organization";
import {
  getCrmKpis,
  getCrmCustomers,
  getCrmSalesPipelineChart,
  getCrmCountriesChart,
  getCrmClosedDeals,
  searchCrmCustomers,
  type CrmKpiResponse,
  type CrmCustomer,
  type CrmPipelineStatus,
  type CrmCountryData,
  type CrmClosedDealData,
} from "@/lib/api";

export default function CrmPage() {
  const [kpis, setKpis] = useState<CrmKpiResponse | null>(null);
  const [customers, setCustomers] = useState<CrmCustomer[]>([]);
  const [allCustomers, setAllCustomers] = useState<CrmCustomer[]>([]);
  const [pipelineData, setPipelineData] = useState<CrmPipelineStatus[]>([]);
  const [countriesData, setCountriesData] = useState<CrmCountryData[]>([]);
  const [closedDealsData, setClosedDealsData] = useState<CrmClosedDealData[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false);
  const [searchTimeout, setSearchTimeout] = useState<NodeJS.Timeout | null>(null);

  useEffect(() => {
    async function fetchData() {
      // Wait for organization UUID to be available
      // Check every 100ms, up to 5 seconds (50 attempts)
      let attempts = 0;
      const maxAttempts = 50;
      
      while (attempts < maxAttempts) {
        const orgUuid = getCurrentOrganizationUuid();
        if (orgUuid) {
          break;
        }
        await new Promise(resolve => setTimeout(resolve, 100));
        attempts++;
      }
      
      // If still no organization UUID after waiting, show error
      const orgUuid = getCurrentOrganizationUuid();
      if (!orgUuid) {
        setError("No organization selected. Please select an organization from the header.");
        setLoading(false);
        return;
      }

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
        setAllCustomers(customersData.customers);
        setPipelineData(pipelineData.statuses);
        setCountriesData(countriesData.countries);
        setClosedDealsData(closedDealsData.deals);
      } catch (err) {
        console.error("Failed to fetch CRM data:", err);
        const errorMessage = err instanceof Error ? err.message : "Failed to load CRM data";
        
        // If it's a missing organization header error, wait a bit and retry once
        if (errorMessage.includes("Missing X-Organization-UUID header")) {
          // Wait a bit more for organization to be set, then retry once
          await new Promise(resolve => setTimeout(resolve, 500));
          const retryOrgUuid = getCurrentOrganizationUuid();
          if (retryOrgUuid) {
            try {
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
              setAllCustomers(customersData.customers);
              setPipelineData(pipelineData.statuses);
              setCountriesData(countriesData.countries);
              setClosedDealsData(closedDealsData.deals);
              setLoading(false);
              return;
            } catch {
              // If retry also fails, show the original error
              setError(errorMessage);
            }
          } else {
            setError("No organization selected. Please select an organization from the header.");
          }
        } else {
          setError(errorMessage);
        }
      } finally {
        setLoading(false);
      }
    }

    fetchData();
  }, []);

  const handleCreateCustomer = () => {
    setIsCreateDialogOpen(true);
  };

  const handleCustomerCreated = async () => {
    // Refresh customers list
    try {
      const customersData = await getCrmCustomers();
      setCustomers(customersData.customers);
      setAllCustomers(customersData.customers);
    } catch (err) {
      console.error("Failed to refresh customers:", err);
    }
  };

  const handleSearch = async (query: string) => {
    // Clear existing timeout
    if (searchTimeout) {
      clearTimeout(searchTimeout);
    }

    // If query is empty, show all customers
    if (!query.trim()) {
      setCustomers(allCustomers);
      return;
    }

    // Debounce search API calls
    const timeout = setTimeout(async () => {
      try {
        const searchResults = await searchCrmCustomers(query.trim());
        setCustomers(searchResults.customers);
      } catch (err) {
        console.error("Failed to search customers:", err);
        
        // If it's an organization membership error, reload the page to refresh organization selection
        if (err instanceof Error && err.message.includes('does not belong to this organization')) {
          window.location.reload();
          return;
        }
        
        // On other errors, show all customers
        setCustomers(allCustomers);
      }
    }, 300); // 300ms debounce

    setSearchTimeout(timeout);
  };

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (searchTimeout) {
        clearTimeout(searchTimeout);
      }
    };
  }, [searchTimeout]);

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
            onSearch={handleSearch}
          />
        </div>

        {/* Create Customer Dialog */}
        <CreateCustomerDialog
          isOpen={isCreateDialogOpen}
          onClose={() => setIsCreateDialogOpen(false)}
          onSuccess={handleCustomerCreated}
        />

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


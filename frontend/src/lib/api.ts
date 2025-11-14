/**
 * API client configuration and utilities
 */

import { getToken } from './auth';
import { getCurrentOrganizationUuid } from './organization';
import { handleNetworkError } from './toast';

/**
 * Get the API base URL from environment variable
 */
function getApiBaseUrl(): string {
  return process.env.NEXT_PUBLIC_API_URL || 'http://localhost:8080';
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  email: string;
  password: string;
}

export interface AuthResponse {
  token: string;
  email: string;
}

export interface ApiError {
  error: string;
}

/**
 * Get the API base URL
 */
export function getApiUrl(): string {
  return getApiBaseUrl();
}

/**
 * Get the full API endpoint URL
 */
export function getApiEndpoint(path: string): string {
  const baseUrl = getApiBaseUrl().replace(/\/$/, '');
  const endpoint = path.startsWith('/') ? path : `/${path}`;
  return `${baseUrl}${endpoint}`;
}

/**
 * Handle organization membership errors by clearing invalid organization UUID
 */
async function handleOrganizationMembershipError(errorMessage: string): Promise<void> {
  if (errorMessage.includes('does not belong to this organization')) {
    const { clearCurrentOrganizationUuid } = await import('./organization');
    clearCurrentOrganizationUuid();
    console.warn('[API] User does not belong to selected organization. Cleared organization UUID.');
  }
}

/**
 * Get headers for API requests (includes auth token and organization UUID)
 * Excludes organization UUID for login and register endpoints
 */
function getApiHeaders(path: string): Record<string, string> {
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  };

  const token = getToken();
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  // Add organization UUID header for all requests except login, register, logout, organizations/list-own, and organizations/create
  const isAuthEndpoint = path === '/api/login' || path === '/api/register';
  const isLogoutEndpoint = path === '/api/logout';
  const isOrgListEndpoint = path === '/api/organizations/list-own';
  const isOrgCreateEndpoint = path === '/api/organizations/create';
  
  if (!isAuthEndpoint && !isLogoutEndpoint && !isOrgListEndpoint && !isOrgCreateEndpoint) {
    const orgUuid = getCurrentOrganizationUuid();
    if (!orgUuid) {
      console.warn(`[API] Missing organization UUID for request to ${path}. Organization may not be selected yet.`);
    } else {
      headers['X-Organization-UUID'] = orgUuid;
    }
  }

  return headers;
}

/**
 * Login user
 */
export async function login(credentials: LoginRequest): Promise<AuthResponse> {
  try {
    const response = await fetch(getApiEndpoint('/api/login'), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(credentials),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        throw new Error(error.error || 'Login failed! Your credentials are wrong.');
      } catch (err) {
        // If we already threw an Error with a specific API error message, re-throw it
        if (err instanceof Error && err.message && err.message !== 'Login failed! Your credentials are wrong.') {
          // Check if it's a known API error message (not a JSON parse error)
          const knownApiErrors = ['Invalid email or password'];
          if (knownApiErrors.some(msg => err.message.includes(msg))) {
            throw err;
          }
        }
        // For JSON parse errors or unknown errors, throw the fallback
        throw new Error('Login failed! Your credentials are wrong.');
      }
    }

    return response.json();
  } catch (error) {
    // Check if it's a network error and show toast
    if (handleNetworkError(error)) {
      throw new Error('Backend API server is not reachable. Try later again.');
    }
    throw error;
  }
}

/**
 * Register user
 */
export async function register(credentials: RegisterRequest): Promise<AuthResponse> {
  try {
    const response = await fetch(getApiEndpoint('/api/register'), {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(credentials),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        throw new Error(error.error || 'Registration failed');
      } catch (err) {
        // If we already threw an Error with a specific API error message, re-throw it
        if (err instanceof Error && err.message && err.message !== 'Registration failed') {
          // Check if it's a known API error message (not a JSON parse error)
          const knownApiErrors = ['Email already exists'];
          if (knownApiErrors.some(msg => err.message.includes(msg))) {
            throw err;
          }
        }
        // For JSON parse errors or unknown errors, throw the fallback
        throw new Error('Registration failed');
      }
    }

    return response.json();
  } catch (error) {
    // Check if it's a network error and show toast
    if (handleNetworkError(error)) {
      throw new Error('Backend API server is not reachable. Try later again.');
    }
    throw error;
  }
}

/**
 * Logout user
 */
export async function logout(userUUID: string): Promise<void> {
  try {
    await fetch(getApiEndpoint('/api/logout'), {
      method: 'POST',
      headers: getApiHeaders('/api/logout'),
      body: JSON.stringify({ user_uuid: userUUID }),
    });
  } catch (error) {
    // Log error but don't throw - logout should succeed even if API call fails
    // Still show toast for network errors
    handleNetworkError(error);
    console.error('Logout API call failed:', error);
  }
}

export interface Organization {
  uuid: string;
  title: string;
  is_admin: boolean;
  license: string;
}

/**
 * Get list of organizations the user belongs to
 */
export async function listOwnOrganizations(): Promise<Organization[]> {
  try {
    const headers = getApiHeaders('/api/organizations/list-own');
    
    // Ensure we have a token before making the request
    if (!headers['Authorization']) {
      throw new Error('Not authenticated. Please log in again.');
    }

    const response = await fetch(getApiEndpoint('/api/organizations/list-own'), {
      method: 'GET',
      headers,
    });

    if (!response.ok) {
      if (response.status === 401) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error || 'Authentication failed. Please log in again.');
      }
      throw new Error('Failed to fetch organizations');
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    console.error('Failed to fetch organizations:', error);
    throw error;
  }
}

export interface CreateOrganizationRequest {
  name: string;
}

/**
 * Create a new organization
 */
export async function createOrganization(request: CreateOrganizationRequest): Promise<Organization> {
  try {
    const headers = getApiHeaders('/api/organizations/create');
    
    // Ensure we have a token before making the request
    if (!headers['Authorization']) {
      throw new Error('Not authenticated. Please log in again.');
    }

    const response = await fetch(getApiEndpoint('/api/organizations/create'), {
      method: 'POST',
      headers,
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        throw new Error(error.error || 'Failed to create organization');
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to create organization');
      }
    }

    const data = await response.json();
    // Map backend response to Organization interface
    return {
      uuid: data.uuid,
      title: data.name,
      is_admin: data.is_admin,
      license: data.license || 'Free',
    };
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    console.error('Failed to create organization:', error);
    throw error;
  }
}

export interface EditWorkflowTitleRequest {
  title: string;
}

export interface EditWorkflowTitleResponse {
  message: string;
  workflow_uuid: string;
  title: string;
}

/**
 * Edit workflow title
 */
export async function editWorkflowTitle(
  workflowUUID: string,
  title: string
): Promise<EditWorkflowTitleResponse> {
  try {
    const response = await fetch(
      getApiEndpoint(`/api/workflows/${workflowUUID}/edit-title`),
      {
        method: 'POST',
        headers: getApiHeaders(`/api/workflows/${workflowUUID}/edit-title`),
        body: JSON.stringify({ title }),
      }
    );

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        throw new Error(error.error || 'Failed to update workflow title');
      } catch {
        throw new Error('Failed to update workflow title');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    console.error('Failed to update workflow title:', error);
    throw error;
  }
}

// CRM API types and functions
export interface CrmKpiResponse {
  total_sales_this_month: number;
  orders_this_month: number;
  orders_last_month: number;
  win_rate_this_month: number;
  avg_days_to_close: number;
  total_users: number;
  open_deals: number;
}

export interface CrmCustomer {
  id: string;
  name: string;
  email: string;
  company: string | null;
  status: string;
  created_at: string;
  last_contact: string | null;
}

// Full customer detail structure (matches backend CrmCustomer)
export interface CrmCustomerDetail {
  uuid: string;
  organization_uuid: string;
  first_name: string;
  last_name: string;
  email: string | null;
  phone_number: string | null;
  user_id: string | null;
  salutation: string | null;
  job_title: string | null;
  department: string | null;
  company_name: string | null;
  fax_number: string | null;
  website_url: string | null;
  gender: string | null;
  created_at: string;
  updated_at: string;
}

export interface CrmCustomerKpis {
  clv: number;
  avg_deal_amount: number;
  org_avg_deal_amount: number;
  last_deal_date: string | null;
  current_sale_status: string;
  source: string;
  assigned_user: string | null;
  days_since_last_contact: number;
  last_interaction_date: string | null;
  created_at: string;
}

export interface CrmCustomerNote {
  uuid: string;
  customer_uuid: string;
  note_text: string;
  author_id: string;
  visible_to_customer: boolean;
  created_at: string;
  updated_at: string;
}

export interface CrmCustomerConversation {
  uuid: string;
  customer_uuid: string;
  message: string;
  source: string; // FROM_TEAM, FROM_CUSTOMER, INTERNAL_NOTE
  channel_uuid: string;
  created_at: string;
}

export interface UpdateCrmCustomerRequest {
  first_name?: string;
  last_name?: string;
  email?: string;
  phone_number?: string;
  user_id?: string;
  salutation?: string;
  job_title?: string;
  department?: string;
  company_name?: string;
  fax_number?: string;
  website_url?: string;
  gender?: string;
}

export interface CrmCustomersResponse {
  customers: CrmCustomer[];
  total: number;
  page: number;
  page_size: number;
  total_pages: number;
}

export interface CrmPipelineStatus {
  status: string;
  count: number;
}

export interface CrmSalesPipelineChartResponse {
  statuses: CrmPipelineStatus[];
}

export interface CrmCountryData {
  country: string;
  count: number;
}

export interface CrmCountriesChartResponse {
  countries: CrmCountryData[];
}

export interface CrmClosedDealData {
  month: string;
  current_year: number;
  previous_year: number;
}

export interface CrmClosedDealsResponse {
  deals: CrmClosedDealData[];
}

export async function getCrmKpis(): Promise<CrmKpiResponse> {
  try {
    const response = await fetch(getApiEndpoint('/api/modules/crm/kpis'), {
      method: 'GET',
      headers: getApiHeaders('/api/modules/crm/kpis'),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to fetch CRM KPIs';
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to fetch CRM KPIs');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    throw error;
  }
}

export async function getCrmCustomers(page: number = 1, pageSize: number = 50): Promise<CrmCustomersResponse> {
  try {
    const params = new URLSearchParams({
      page: page.toString(),
      page_size: pageSize.toString(),
    });
    const response = await fetch(getApiEndpoint(`/api/modules/crm/customers?${params.toString()}`), {
      method: 'GET',
      headers: getApiHeaders('/api/modules/crm/customers'),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to fetch CRM customers';
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to fetch CRM customers');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    throw error;
  }
}

export async function getCrmSalesPipelineChart(): Promise<CrmSalesPipelineChartResponse> {
  try {
    const response = await fetch(
      getApiEndpoint('/api/modules/crm/sales-pipeline-chart'),
      {
        method: 'GET',
        headers: getApiHeaders('/api/modules/crm/sales-pipeline-chart'),
      }
    );

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to fetch sales pipeline chart data';
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to fetch sales pipeline chart data');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    throw error;
  }
}

export async function getCrmCountriesChart(): Promise<CrmCountriesChartResponse> {
  try {
    const response = await fetch(
      getApiEndpoint('/api/modules/crm/countries-chart'),
      {
        method: 'GET',
        headers: getApiHeaders('/api/modules/crm/countries-chart'),
      }
    );

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to fetch countries chart data';
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to fetch countries chart data');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    throw error;
  }
}

export async function getCrmClosedDeals(): Promise<CrmClosedDealsResponse> {
  try {
    const response = await fetch(
      getApiEndpoint('/api/modules/crm/closed-deals'),
      {
        method: 'GET',
        headers: getApiHeaders('/api/modules/crm/closed-deals'),
      }
    );

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to fetch closed deals data';
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to fetch closed deals data');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    throw error;
  }
}

export interface CreateCrmCustomerRequest {
  first_name: string;
  last_name: string;
  email?: string;
  phone_number?: string;
  user_id?: string;
  salutation?: string;
  job_title?: string;
  department?: string;
  company_name?: string;
  fax_number?: string;
  website_url?: string;
  gender?: string;
}

export interface CreateCrmCustomerResponse {
  uuid: string;
  message: string;
}

export async function createCrmCustomer(
  request: CreateCrmCustomerRequest
): Promise<CreateCrmCustomerResponse> {
  try {
    const response = await fetch(
      getApiEndpoint('/api/modules/crm/customers'),
      {
        method: 'POST',
        headers: getApiHeaders('/api/modules/crm/customers'),
        body: JSON.stringify(request),
      }
    );

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to create customer';
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to create customer');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    console.error('Failed to create customer:', error);
    throw error;
  }
}

export async function searchCrmCustomers(
  query: string
): Promise<CrmCustomersResponse> {
  try {
    const response = await fetch(
      getApiEndpoint(`/api/modules/crm/customers/search?q=${encodeURIComponent(query)}`),
      {
        method: 'GET',
        headers: getApiHeaders('/api/modules/crm/customers/search'),
      }
    );

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to search customers';
        
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to search customers');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    console.error('Failed to search customers:', error);
    throw error;
  }
}

export async function getCrmCustomer(uuid: string): Promise<CrmCustomerDetail> {
  try {
    const response = await fetch(getApiEndpoint(`/api/modules/crm/customers/${uuid}`), {
      method: 'GET',
      headers: getApiHeaders('/api/modules/crm/customers'),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to fetch customer';
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to fetch customer');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    console.error('Failed to fetch customer:', error);
    throw error;
  }
}

export async function getCrmCustomerKpis(uuid: string): Promise<CrmCustomerKpis> {
  try {
    const response = await fetch(getApiEndpoint(`/api/modules/crm/customers/${uuid}/kpis`), {
      method: 'GET',
      headers: getApiHeaders('/api/modules/crm/customers'),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to fetch customer KPIs';
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to fetch customer KPIs');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    console.error('Failed to fetch customer KPIs:', error);
    throw error;
  }
}

export async function getCrmCustomerNotes(uuid: string): Promise<CrmCustomerNote[]> {
  try {
    const response = await fetch(getApiEndpoint(`/api/modules/crm/customers/${uuid}/notes`), {
      method: 'GET',
      headers: getApiHeaders('/api/modules/crm/customers'),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to fetch customer notes';
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to fetch customer notes');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    console.error('Failed to fetch customer notes:', error);
    throw error;
  }
}

export async function getCrmCustomerConversations(uuid: string): Promise<CrmCustomerConversation[]> {
  try {
    const response = await fetch(getApiEndpoint(`/api/modules/crm/customers/${uuid}/conversations`), {
      method: 'GET',
      headers: getApiHeaders('/api/modules/crm/customers'),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to fetch customer conversations';
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to fetch customer conversations');
      }
    }

    return response.json();
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    console.error('Failed to fetch customer conversations:', error);
    throw error;
  }
}

export async function updateCrmCustomer(uuid: string, data: UpdateCrmCustomerRequest): Promise<void> {
  try {
    const response = await fetch(getApiEndpoint(`/api/modules/crm/customers/${uuid}`), {
      method: 'PUT',
      headers: getApiHeaders('/api/modules/crm/customers'),
      body: JSON.stringify(data),
    });

    if (!response.ok) {
      try {
        const error: ApiError = await response.json();
        const errorMessage = error.error || 'Failed to update customer';
        await handleOrganizationMembershipError(errorMessage);
        throw new Error(errorMessage);
      } catch (err) {
        if (err instanceof Error) {
          throw err;
        }
        throw new Error('Failed to update customer');
      }
    }
  } catch (error) {
    if (handleNetworkError(error)) {
      throw new Error('Network error: Unable to connect to the server');
    }
    console.error('Failed to update customer:', error);
    throw error;
  }
}


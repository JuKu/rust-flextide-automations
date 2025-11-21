"use client";

import { useState, useEffect } from "react";
import { updateChromaDatabase, getChromaDatabase, testChromaConnection, ChromaCredentials, UpdateChromaDatabaseRequest } from "@/lib/api";
import { showToast } from "@/lib/toast";

interface EditChromaDatabaseDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
  databaseUuid: string | null;
}

export function EditChromaDatabaseDialog({
  isOpen,
  onClose,
  onSuccess,
  databaseUuid,
}: EditChromaDatabaseDialogProps) {
  const [name, setName] = useState("");
  const [baseUrl, setBaseUrl] = useState("https://api.trychroma.com");
  const [securedMode, setSecuredMode] = useState(true);
  const [authMethod, setAuthMethod] = useState("token");
  const [tokenTransportHeader, setTokenTransportHeader] = useState("x-chroma-token");
  const [tokenPrefix, setTokenPrefix] = useState("");
  const [authToken, setAuthToken] = useState("");
  const [tenantName, setTenantName] = useState("default_tenant");
  const [databaseName, setDatabaseName] = useState("default_database");
  const [additionalHeaders, setAdditionalHeaders] = useState("");
  const [apiVersion, setApiVersion] = useState("v2");
  
  const [loading, setLoading] = useState(false);
  const [loadingData, setLoadingData] = useState(false);
  const [testingConnection, setTestingConnection] = useState(false);
  const [connectionTested, setConnectionTested] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load database data when dialog opens
  useEffect(() => {
    if (isOpen && databaseUuid) {
      loadDatabaseData();
    } else if (!isOpen) {
      // Reset form when dialog closes
      setName("");
      setBaseUrl("https://api.trychroma.com");
      setSecuredMode(true);
      setAuthMethod("token");
      setTokenTransportHeader("x-chroma-token");
      setTokenPrefix("");
      setAuthToken("");
      setTenantName("default_tenant");
      setDatabaseName("default_database");
      setAdditionalHeaders("");
      setApiVersion("v2");
      setConnectionTested(false);
      setError(null);
    }
  }, [isOpen, databaseUuid]);

  const loadDatabaseData = async () => {
    if (!databaseUuid) return;

    try {
      setLoadingData(true);
      setError(null);
      const data = await getChromaDatabase(databaseUuid);
      
      setName(data.name);
      setBaseUrl(data.credentials.base_url);
      setSecuredMode(data.credentials.secured_mode);
      setAuthMethod(data.credentials.auth_method);
      setTokenTransportHeader(data.credentials.token_transport_header);
      setTokenPrefix(data.credentials.token_prefix || "");
      setAuthToken(data.credentials.auth_token);
      setTenantName(data.credentials.tenant_name);
      setDatabaseName(data.credentials.database_name);
      setApiVersion(data.credentials.api_version || "v2");
      
      // Format additional headers for textarea
      if (data.credentials.additional_headers && data.credentials.additional_headers.length > 0) {
        const headersText = data.credentials.additional_headers
          .map(([key, value]) => `${key}: ${value}`)
          .join("\n");
        setAdditionalHeaders(headersText);
      } else {
        setAdditionalHeaders("");
      }
    } catch (err) {
      console.error("Failed to load database data:", err);
      const errorMessage = err instanceof Error ? err.message : "Failed to load database data";
      setError(errorMessage);
      showToast(errorMessage, "error");
    } finally {
      setLoadingData(false);
    }
  };

  // Handle secured mode change
  useEffect(() => {
    if (!securedMode) {
      setAuthMethod("none");
    } else if (authMethod === "none") {
      setAuthMethod("token");
    }
  }, [securedMode, authMethod]);

  // Handle ESC key to close dialog
  useEffect(() => {
    function handleEscape(e: KeyboardEvent) {
      if (e.key === "Escape" && !loading && !testingConnection && !loadingData) {
        onClose();
      }
    }

    if (isOpen) {
      document.addEventListener("keydown", handleEscape);
    }

    return () => {
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen, loading, testingConnection, loadingData, onClose]);

  // Parse additional headers
  const parseAdditionalHeaders = (text: string): Array<[string, string]> => {
    if (!text.trim()) return [];
    
    const lines = text.split("\n").filter(line => line.trim());
    const headers: Array<[string, string]> = [];
    
    for (const line of lines) {
      const colonIndex = line.indexOf(":");
      if (colonIndex === -1) {
        throw new Error(`Invalid header format: "${line}". Expected format: "Header-Name: value"`);
      }
      const headerName = line.substring(0, colonIndex).trim();
      const headerValue = line.substring(colonIndex + 1).trim();
      if (!headerName || !headerValue) {
        throw new Error(`Invalid header format: "${line}". Both name and value are required.`);
      }
      headers.push([headerName, headerValue]);
    }
    
    return headers;
  };

  const handleTestConnection = async () => {
    setError(null);
    setTestingConnection(true);

    // Validate required fields
    if (!baseUrl.trim()) {
      setError("Base URL is required");
      setTestingConnection(false);
      return;
    }

    if (!tenantName.trim()) {
      setError("Tenant name is required");
      setTestingConnection(false);
      return;
    }

    if (!databaseName.trim()) {
      setError("Database name is required");
      setTestingConnection(false);
      return;
    }

    if (authMethod !== "none" && !authToken.trim()) {
      setError("Auth token is required");
      setTestingConnection(false);
      return;
    }

    if (authMethod === "basic_auth" && !authToken.includes(":")) {
      setError("Auth token must be in format 'username:password' for Basic Auth");
      setTestingConnection(false);
      return;
    }

    // Parse additional headers
    let parsedHeaders: Array<[string, string]> = [];
    if (additionalHeaders.trim()) {
      try {
        parsedHeaders = parseAdditionalHeaders(additionalHeaders);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Invalid additional headers format");
        setTestingConnection(false);
        return;
      }
    }

    try {
      const credentials: ChromaCredentials = {
        base_url: baseUrl.trim(),
        secured_mode: securedMode,
        auth_method: authMethod,
        token_transport_header: tokenTransportHeader.trim(),
        token_prefix: tokenPrefix.trim(),
        auth_token: authToken,
        tenant_name: tenantName.trim(),
        database_name: databaseName.trim(),
        additional_headers: parsedHeaders,
        api_version: apiVersion,
      };

      await testChromaConnection({ credentials });
      setConnectionTested(true);
      showToast("Connection test successful", "success");
    } catch (err) {
      console.error("Failed to test connection:", err);
      const errorMessage = err instanceof Error ? err.message : "Failed to test connection";
      setError(errorMessage);
      setConnectionTested(false);
      showToast(errorMessage, "error");
    } finally {
      setTestingConnection(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!databaseUuid) {
      setError("Database UUID is missing");
      return;
    }

    if (!name.trim()) {
      setError("Name is required");
      return;
    }

    if (!connectionTested) {
      setError("Please test the connection before saving");
      return;
    }

    // Validate required fields
    if (!baseUrl.trim()) {
      setError("Base URL is required");
      return;
    }

    if (!tenantName.trim()) {
      setError("Tenant name is required");
      return;
    }

    if (!databaseName.trim()) {
      setError("Database name is required");
      return;
    }

    if (authMethod !== "none" && !authToken.trim()) {
      setError("Auth token is required");
      return;
    }

    if (authMethod === "basic_auth" && !authToken.includes(":")) {
      setError("Auth token must be in format 'username:password' for Basic Auth");
      return;
    }

    // Parse additional headers
    let parsedHeaders: Array<[string, string]> = [];
    if (additionalHeaders.trim()) {
      try {
        parsedHeaders = parseAdditionalHeaders(additionalHeaders);
      } catch (err) {
        setError(err instanceof Error ? err.message : "Invalid additional headers format");
        return;
      }
    }

    try {
      setLoading(true);

      const credentials: ChromaCredentials = {
        base_url: baseUrl.trim(),
        secured_mode: securedMode,
        auth_method: authMethod,
        token_transport_header: tokenTransportHeader.trim(),
        token_prefix: tokenPrefix.trim(),
        auth_token: authToken,
        tenant_name: tenantName.trim(),
        database_name: databaseName.trim(),
        additional_headers: parsedHeaders,
        api_version: apiVersion,
      };

      const request: UpdateChromaDatabaseRequest = {
        name: name.trim(),
        credentials,
      };

      await updateChromaDatabase(databaseUuid, request);
      showToast("Chroma database connection updated successfully", "success");
      onSuccess();
    } catch (err) {
      console.error("Failed to update Chroma database:", err);
      const errorMessage = err instanceof Error ? err.message : "Failed to update Chroma database";
      setError(errorMessage);
    } finally {
      setLoading(false);
    }
  };

  const handleClose = () => {
    if (!loading && !testingConnection && !loadingData) {
      onClose();
    }
  };

  if (!isOpen || !databaseUuid) return null;

  const isSecuredFieldsDisabled = !securedMode;

  return (
    <div 
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
      onClick={handleClose}
    >
      <div 
        className="bg-flextide-neutral-panel-bg rounded-lg shadow-xl w-full max-w-2xl mx-4 max-h-[90vh] overflow-y-auto"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="p-6 border-b border-flextide-neutral-border">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            Edit Chroma Database
          </h2>
          <p className="text-sm text-flextide-neutral-text-medium mt-1">
            Update Chroma vector database connection settings
          </p>
        </div>

        {loadingData ? (
          <div className="p-6 text-center">
            <p className="text-flextide-neutral-text-medium">Loading database data...</p>
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="p-6">
            {error && (
              <div className="mb-4 p-3 bg-flextide-error/10 border border-flextide-error rounded-md text-flextide-error text-sm">
                {error}
              </div>
            )}

            <div className="mb-4">
              <label
                htmlFor="name"
                className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
              >
                Connection Name <span className="text-flextide-error">*</span>
              </label>
              <input
                id="name"
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="e.g., Production Chroma, Development Chroma"
                className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
                required
                disabled={loading || testingConnection || loadingData}
              />
            </div>

            <div className="mb-4">
              <label
                htmlFor="api-version"
                className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
              >
                API Version <span className="text-flextide-error">*</span>
              </label>
              <select
                id="api-version"
                value={apiVersion}
                onChange={(e) => setApiVersion(e.target.value)}
                className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
                required
                disabled={loading || testingConnection || loadingData}
              >
                <option value="v2">v2</option>
              </select>
            </div>

            <div className="mb-4">
              <label
                htmlFor="base-url"
                className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
              >
                Base URL <span className="text-flextide-error">*</span>
              </label>
              <div className="relative">
                <input
                  id="base-url"
                  type="text"
                  value={baseUrl}
                  onChange={(e) => setBaseUrl(e.target.value)}
                  placeholder="https://api.trychroma.com"
                  className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
                  required
                  disabled={loading || testingConnection || loadingData}
                />
                <div className="group relative">
                  <svg
                    className="absolute right-2 top-2.5 h-4 w-4 text-flextide-neutral-text-medium cursor-help"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      strokeWidth={2}
                      d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    />
                  </svg>
                  <div className="absolute right-0 top-6 w-64 p-2 bg-flextide-neutral-text-dark text-white text-xs rounded-md opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-10">
                    Also supports http://&lt;IP&gt;:&lt;Port&gt; for local deployments
                  </div>
                </div>
              </div>
            </div>

            <div className="mb-4">
              <label className="flex items-center">
                <input
                  type="checkbox"
                  checked={securedMode}
                  onChange={(e) => setSecuredMode(e.target.checked)}
                  className="mr-2"
                  disabled={loading || testingConnection || loadingData}
                />
                <span className="text-sm font-medium text-flextide-neutral-text-dark">
                  Secured Mode (Authentication Enabled)
                </span>
              </label>
            </div>

            <div className="mb-4">
              <label
                htmlFor="auth-method"
                className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
              >
                Authentication Method <span className="text-flextide-error">*</span>
              </label>
              <select
                id="auth-method"
                value={authMethod}
                onChange={(e) => setAuthMethod(e.target.value)}
                className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent disabled:bg-flextide-neutral-light-bg disabled:text-flextide-neutral-text-medium"
                required
                disabled={loading || testingConnection || loadingData || isSecuredFieldsDisabled}
              >
                <option value="token">Token based Authentication</option>
                <option value="basic_auth">Basic Auth</option>
                <option value="none">None</option>
              </select>
            </div>

            {authMethod !== "none" && (
              <>
                <div className="mb-4">
                  <label
                    htmlFor="token-transport-header"
                    className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
                  >
                    Token Transport Header <span className="text-flextide-error">*</span>
                  </label>
                  <input
                    id="token-transport-header"
                    type="text"
                    value={tokenTransportHeader}
                    onChange={(e) => setTokenTransportHeader(e.target.value)}
                    placeholder="x-chroma-token"
                    className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent disabled:bg-flextide-neutral-light-bg disabled:text-flextide-neutral-text-medium"
                    required
                    disabled={loading || testingConnection || loadingData || isSecuredFieldsDisabled}
                  />
                </div>

                <div className="mb-4">
                  <label
                    htmlFor="token-prefix"
                    className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
                  >
                    Token Prefix (Optional)
                  </label>
                  <div className="relative">
                    <input
                      id="token-prefix"
                      type="text"
                      value={tokenPrefix}
                      onChange={(e) => setTokenPrefix(e.target.value)}
                      placeholder="Bearer "
                      className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent disabled:bg-flextide-neutral-light-bg disabled:text-flextide-neutral-text-medium"
                      disabled={loading || testingConnection || loadingData || isSecuredFieldsDisabled}
                    />
                    <div className="group relative">
                      <svg
                        className="absolute right-2 top-2.5 h-4 w-4 text-flextide-neutral-text-medium cursor-help"
                        fill="none"
                        stroke="currentColor"
                        viewBox="0 0 24 24"
                      >
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                        />
                      </svg>
                      <div className="absolute right-0 top-6 w-48 p-2 bg-flextide-neutral-text-dark text-white text-xs rounded-md opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-10">
                        Can be values like &quot;Bearer &quot; for Authorization header
                      </div>
                    </div>
                  </div>
                </div>

                <div className="mb-4">
                  <label
                    htmlFor="auth-token"
                    className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
                  >
                    {authMethod === "basic_auth" ? "API Key (username:password)" : "API Key"} <span className="text-flextide-error">*</span>
                  </label>
                  <input
                    id="auth-token"
                    type="password"
                    value={authToken}
                    onChange={(e) => setAuthToken(e.target.value)}
                    placeholder={authMethod === "basic_auth" ? "username:password" : "Enter API key"}
                    className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent disabled:bg-flextide-neutral-light-bg disabled:text-flextide-neutral-text-medium"
                    required
                    disabled={loading || testingConnection || loadingData || isSecuredFieldsDisabled}
                  />
                  {authMethod === "basic_auth" && (
                    <p className="mt-1 text-xs text-flextide-neutral-text-medium">
                      Format: username:password (colon required)
                    </p>
                  )}
                </div>
              </>
            )}

            <div className="mb-4">
              <label
                htmlFor="tenant-name"
                className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
              >
                Tenant Name <span className="text-flextide-error">*</span>
              </label>
              <input
                id="tenant-name"
                type="text"
                value={tenantName}
                onChange={(e) => setTenantName(e.target.value)}
                placeholder="default_tenant"
                className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
                required
                disabled={loading || testingConnection || loadingData}
              />
            </div>

            <div className="mb-4">
              <label
                htmlFor="database-name"
                className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
              >
                Database Name <span className="text-flextide-error">*</span>
              </label>
              <input
                id="database-name"
                type="text"
                value={databaseName}
                onChange={(e) => setDatabaseName(e.target.value)}
                placeholder="default_database"
                className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent"
                required
                disabled={loading || testingConnection || loadingData}
              />
            </div>

            <div className="mb-6">
              <label
                htmlFor="additional-headers"
                className="block text-sm font-medium text-flextide-neutral-text-dark mb-2"
              >
                Additional Headers (Optional)
              </label>
              <textarea
                id="additional-headers"
                value={additionalHeaders}
                onChange={(e) => setAdditionalHeaders(e.target.value)}
                placeholder='Header-Name: value&#10;Another-Header: another-value'
                rows={4}
                className="w-full px-3 py-2 border border-flextide-neutral-border rounded-md focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent focus:border-transparent font-mono text-sm"
                disabled={loading || testingConnection || loadingData}
              />
              <p className="mt-1 text-xs text-flextide-neutral-text-medium">
                One header per line in format: Header-Name: value
              </p>
            </div>

            <div className="flex justify-end gap-3">
              <button
                type="button"
                onClick={handleClose}
                disabled={loading || testingConnection || loadingData}
                className="px-4 py-2 text-sm font-medium text-flextide-neutral-text-dark bg-flextide-neutral-light-bg border border-flextide-neutral-border rounded-md hover:bg-flextide-neutral-border transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Close
              </button>
              <button
                type="button"
                onClick={handleTestConnection}
                disabled={loading || testingConnection || loadingData}
                className="px-4 py-2 text-sm font-medium text-flextide-neutral-text-dark bg-flextide-neutral-light-bg border border-flextide-neutral-border rounded-md hover:bg-flextide-neutral-border transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {testingConnection ? "Testing..." : "Test Connection"}
              </button>
              <div className="relative group">
                <button
                  type="submit"
                  disabled={loading || testingConnection || loadingData || !connectionTested}
                  className="px-4 py-2 text-sm font-medium text-white bg-flextide-primary rounded-md hover:bg-flextide-primary-accent transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {loading ? "Saving..." : "Save"}
                </button>
                {!connectionTested && !loading && !testingConnection && !loadingData && (
                  <div className="absolute right-0 bottom-full mb-2 w-64 p-2 bg-flextide-neutral-text-dark text-white text-xs rounded-md opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-10">
                    Please test the connection first before saving
                  </div>
                )}
              </div>
            </div>
          </form>
        )}
      </div>
    </div>
  );
}


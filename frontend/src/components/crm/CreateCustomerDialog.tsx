"use client";

import { useState, useEffect, useRef } from "react";
import { createCrmCustomer } from "@/lib/api";

interface CreateCustomerDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

export function CreateCustomerDialog({
  isOpen,
  onClose,
  onSuccess,
}: CreateCustomerDialogProps) {
  const [formData, setFormData] = useState({
    first_name: "",
    last_name: "",
    email: "",
    phone_number: "",
    salutation: "",
    job_title: "",
    department: "",
    company_name: "",
    fax_number: "",
    website_url: "",
    gender: "",
  });
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const dialogRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    function handleEscape(e: KeyboardEvent) {
      if (e.key === "Escape") {
        onClose();
      }
    }

    if (isOpen) {
      document.addEventListener("keydown", handleEscape);
    }

    return () => {
      document.removeEventListener("keydown", handleEscape);
    };
  }, [isOpen, onClose]);

  useEffect(() => {
    if (!isOpen) {
      // Reset form when dialog closes
      setFormData({
        first_name: "",
        last_name: "",
        email: "",
        phone_number: "",
        salutation: "",
        job_title: "",
        department: "",
        company_name: "",
        fax_number: "",
        website_url: "",
        gender: "",
      });
      setErrors({});
    }
  }, [isOpen]);

  const handleChange = (
    e: React.ChangeEvent<HTMLInputElement | HTMLSelectElement>
  ) => {
    const { name, value } = e.target;
    setFormData((prev) => ({ ...prev, [name]: value }));
    // Clear error for this field
    if (errors[name]) {
      setErrors((prev) => {
        const newErrors = { ...prev };
        delete newErrors[name];
        return newErrors;
      });
    }
  };

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.first_name.trim()) {
      newErrors.first_name = "First name is required";
    }

    if (!formData.last_name.trim()) {
      newErrors.last_name = "Last name is required";
    }

    if (formData.email && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.email)) {
      newErrors.email = "Invalid email format";
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validate()) {
      return;
    }

    setIsSubmitting(true);

    try {
      await createCrmCustomer({
        first_name: formData.first_name.trim(),
        last_name: formData.last_name.trim(),
        email: formData.email.trim() || undefined,
        phone_number: formData.phone_number.trim() || undefined,
        salutation: formData.salutation.trim() || undefined,
        job_title: formData.job_title.trim() || undefined,
        department: formData.department.trim() || undefined,
        company_name: formData.company_name.trim() || undefined,
        fax_number: formData.fax_number.trim() || undefined,
        website_url: formData.website_url.trim() || undefined,
        gender: formData.gender.trim() || undefined,
      });

      onSuccess();
      onClose();
    } catch (error) {
      console.error("Failed to create customer:", error);
      setErrors({
        submit: error instanceof Error ? error.message : "Failed to create customer",
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black bg-opacity-50 p-4">
      <div
        ref={dialogRef}
        className="w-full max-w-2xl max-h-[90vh] overflow-y-auto rounded-lg bg-flextide-neutral-panel-bg border border-flextide-neutral-border shadow-xl"
      >
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-flextide-neutral-border sticky top-0 bg-flextide-neutral-panel-bg">
          <h2 className="text-xl font-semibold text-flextide-neutral-text-dark">
            Create New Customer
          </h2>
          <button
            onClick={onClose}
            className="p-1 rounded-md text-flextide-neutral-text-medium hover:bg-flextide-neutral-light-bg transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
            aria-label="Close dialog"
          >
            <svg
              className="w-6 h-6"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M6 18L18 6M6 6l12 12"
              />
            </svg>
          </button>
        </div>

        {/* Content */}
        <form onSubmit={handleSubmit}>
          <div className="p-6 space-y-4">
            {errors.submit && (
              <div className="p-3 rounded-md bg-flextide-error/10 border border-flextide-error text-flextide-error text-sm">
                {errors.submit}
              </div>
            )}

            <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
              {/* First Name */}
              <div>
                <label
                  htmlFor="first_name"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
                >
                  First Name <span className="text-flextide-error">*</span>
                </label>
                <input
                  type="text"
                  id="first_name"
                  name="first_name"
                  value={formData.first_name}
                  onChange={handleChange}
                  className={`w-full px-3 py-2 rounded-md border ${
                    errors.first_name
                      ? "border-flextide-error"
                      : "border-flextide-neutral-border"
                  } bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent`}
                  required
                />
                {errors.first_name && (
                  <p className="mt-1 text-sm text-flextide-error">
                    {errors.first_name}
                  </p>
                )}
              </div>

              {/* Last Name */}
              <div>
                <label
                  htmlFor="last_name"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
                >
                  Last Name <span className="text-flextide-error">*</span>
                </label>
                <input
                  type="text"
                  id="last_name"
                  name="last_name"
                  value={formData.last_name}
                  onChange={handleChange}
                  className={`w-full px-3 py-2 rounded-md border ${
                    errors.last_name
                      ? "border-flextide-error"
                      : "border-flextide-neutral-border"
                  } bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent`}
                  required
                />
                {errors.last_name && (
                  <p className="mt-1 text-sm text-flextide-error">
                    {errors.last_name}
                  </p>
                )}
              </div>

              {/* Email */}
              <div>
                <label
                  htmlFor="email"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
                >
                  Email
                </label>
                <input
                  type="email"
                  id="email"
                  name="email"
                  value={formData.email}
                  onChange={handleChange}
                  className={`w-full px-3 py-2 rounded-md border ${
                    errors.email
                      ? "border-flextide-error"
                      : "border-flextide-neutral-border"
                  } bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent`}
                />
                {errors.email && (
                  <p className="mt-1 text-sm text-flextide-error">
                    {errors.email}
                  </p>
                )}
              </div>

              {/* Phone Number */}
              <div>
                <label
                  htmlFor="phone_number"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
                >
                  Phone Number
                </label>
                <input
                  type="tel"
                  id="phone_number"
                  name="phone_number"
                  value={formData.phone_number}
                  onChange={handleChange}
                  className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                />
              </div>

              {/* Salutation */}
              <div>
                <label
                  htmlFor="salutation"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
                >
                  Salutation
                </label>
                <select
                  id="salutation"
                  name="salutation"
                  value={formData.salutation}
                  onChange={handleChange}
                  className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                >
                  <option value="">Select...</option>
                  <option value="Mr.">Mr.</option>
                  <option value="Mrs.">Mrs.</option>
                  <option value="Ms.">Ms.</option>
                  <option value="Dr.">Dr.</option>
                  <option value="Prof.">Prof.</option>
                </select>
              </div>

              {/* Gender */}
              <div>
                <label
                  htmlFor="gender"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
                >
                  Gender
                </label>
                <select
                  id="gender"
                  name="gender"
                  value={formData.gender}
                  onChange={handleChange}
                  className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                >
                  <option value="">Select...</option>
                  <option value="Male">Male</option>
                  <option value="Female">Female</option>
                  <option value="Other">Other</option>
                  <option value="Prefer not to say">Prefer not to say</option>
                </select>
              </div>

              {/* Job Title */}
              <div>
                <label
                  htmlFor="job_title"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
                >
                  Job Title
                </label>
                <input
                  type="text"
                  id="job_title"
                  name="job_title"
                  value={formData.job_title}
                  onChange={handleChange}
                  className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                />
              </div>

              {/* Department */}
              <div>
                <label
                  htmlFor="department"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
                >
                  Department
                </label>
                <input
                  type="text"
                  id="department"
                  name="department"
                  value={formData.department}
                  onChange={handleChange}
                  className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                />
              </div>

              {/* Company Name */}
              <div>
                <label
                  htmlFor="company_name"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
                >
                  Company Name
                </label>
                <input
                  type="text"
                  id="company_name"
                  name="company_name"
                  value={formData.company_name}
                  onChange={handleChange}
                  className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                />
              </div>

              {/* Fax Number */}
              <div>
                <label
                  htmlFor="fax_number"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
                >
                  Fax Number
                </label>
                <input
                  type="tel"
                  id="fax_number"
                  name="fax_number"
                  value={formData.fax_number}
                  onChange={handleChange}
                  className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                />
              </div>

              {/* Website URL */}
              <div className="sm:col-span-2">
                <label
                  htmlFor="website_url"
                  className="block text-sm font-medium text-flextide-neutral-text-dark mb-1"
                >
                  Website URL
                </label>
                <input
                  type="url"
                  id="website_url"
                  name="website_url"
                  value={formData.website_url}
                  onChange={handleChange}
                  className="w-full px-3 py-2 rounded-md border border-flextide-neutral-border bg-flextide-neutral-panel-bg text-flextide-neutral-text-dark focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
                  placeholder="https://example.com"
                />
              </div>
            </div>
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-flextide-neutral-border">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-flextide-neutral-text-dark bg-flextide-neutral-light-bg hover:bg-flextide-neutral-border rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent"
              disabled={isSubmitting}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="px-4 py-2 text-sm font-medium text-white bg-flextide-primary hover:bg-flextide-primary-accent rounded-md transition-colors focus:outline-none focus:ring-2 focus:ring-flextide-primary-accent disabled:opacity-50 disabled:cursor-not-allowed"
              disabled={isSubmitting}
            >
              {isSubmitting ? "Creating..." : "Create Customer"}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}


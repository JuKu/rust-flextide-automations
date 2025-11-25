/**
 * Tests for AddChromaDatabaseDialog component
 */

import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { AddChromaDatabaseDialog } from '../AddChromaDatabaseDialog';
import * as api from '@/lib/api';
import * as toast from '@/lib/toast';

// Mock API functions
jest.mock('@/lib/api', () => ({
  testChromaConnection: jest.fn(),
  createChromaDatabase: jest.fn(),
}));

// Mock toast functions
jest.mock('@/lib/toast', () => ({
  showToast: jest.fn(),
}));

describe('AddChromaDatabaseDialog', () => {
  const mockOnClose = jest.fn();
  const mockOnSuccess = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should not render when isOpen is false', () => {
      render(
        <AddChromaDatabaseDialog
          isOpen={false}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      expect(screen.queryByText('Add Chroma Database')).not.toBeInTheDocument();
    });

    it('should render when isOpen is true', () => {
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      expect(screen.getByText('Add Chroma Database')).toBeInTheDocument();
      expect(screen.getByText('Configure a new Chroma vector database connection')).toBeInTheDocument();
    });

    it('should render all form fields with default values', () => {
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      expect(screen.getByLabelText(/connection name/i)).toHaveValue('');
      expect(screen.getByLabelText(/base url/i)).toHaveValue('https://api.trychroma.com');
      expect(screen.getByLabelText(/api version/i)).toHaveValue('v2');
      expect(screen.getByLabelText(/tenant name/i)).toHaveValue('default_tenant');
      expect(screen.getByLabelText(/database name/i)).toHaveValue('default_database');
      
      const securedModeCheckbox = screen.getByLabelText(/secured mode/i);
      expect(securedModeCheckbox).toBeChecked();
    });
  });

  describe('Form Interactions', () => {
    it('should update form fields when user types', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const nameInput = screen.getByLabelText(/connection name/i);
      await user.type(nameInput, 'Test Database');

      expect(nameInput).toHaveValue('Test Database');
    });

    it('should toggle secured mode checkbox', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const securedModeCheckbox = screen.getByLabelText(/secured mode/i);
      expect(securedModeCheckbox).toBeChecked();

      await user.click(securedModeCheckbox);

      expect(securedModeCheckbox).not.toBeChecked();
    });

    it('should disable auth fields when secured mode is disabled', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const securedModeCheckbox = screen.getByLabelText(/secured mode/i);
      await user.click(securedModeCheckbox);

      const authMethodSelect = screen.getByLabelText(/authentication method/i);
      expect(authMethodSelect).toBeDisabled();
      expect(authMethodSelect).toHaveValue('none');
    });

    it('should set auth_method to "token" when secured mode is enabled', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const securedModeCheckbox = screen.getByLabelText(/secured mode/i);
      
      // Disable and re-enable
      await user.click(securedModeCheckbox);
      await user.click(securedModeCheckbox);

      const authMethodSelect = screen.getByLabelText(/authentication method/i);
      expect(authMethodSelect).toHaveValue('token');
    });

    it('should show auth fields when auth_method is not "none"', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const authMethodSelect = screen.getByLabelText(/authentication method/i);
      await user.selectOptions(authMethodSelect, 'token');

      expect(screen.getByLabelText(/token transport header/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/token prefix/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/api key/i)).toBeInTheDocument();
    });

    it('should hide auth fields when auth_method is "none"', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const securedModeCheckbox = screen.getByLabelText(/secured mode/i);
      await user.click(securedModeCheckbox);

      expect(screen.queryByLabelText(/token transport header/i)).not.toBeInTheDocument();
      expect(screen.queryByLabelText(/api key/i)).not.toBeInTheDocument();
    });

    it('should show basic_auth format hint when auth_method is basic_auth', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const authMethodSelect = screen.getByLabelText(/authentication method/i);
      await user.selectOptions(authMethodSelect, 'basic_auth');

      // There are multiple elements with "username:password" text (label and hint)
      expect(screen.getAllByText(/username:password/i).length).toBeGreaterThan(0);
    });
  });

  describe('Connection Testing', () => {
    it('should test connection successfully', async () => {
      const user = userEvent.setup();
      (api.testChromaConnection as jest.Mock).mockResolvedValueOnce({
        success: true,
        message: 'Connection test successful',
      });

      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const nameInput = screen.getByLabelText(/connection name/i);
      await user.type(nameInput, 'Test DB');

      // Fill in required auth token field
      const authTokenInput = screen.getByLabelText(/api key/i);
      await user.type(authTokenInput, 'test-token');

      const testButton = screen.getByRole('button', { name: /test connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(api.testChromaConnection).toHaveBeenCalled();
        expect(toast.showToast).toHaveBeenCalledWith('Connection test successful', 'success');
      });

      const saveButton = screen.getByRole('button', { name: /save/i });
      expect(saveButton).not.toBeDisabled();
    });

    it('should show error when connection test fails', async () => {
      const user = userEvent.setup();
      const errorMessage = 'Invalid API key or authentication failed. Please check your credentials.';
      (api.testChromaConnection as jest.Mock).mockRejectedValueOnce(new Error(errorMessage));

      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const nameInput = screen.getByLabelText(/connection name/i);
      await user.type(nameInput, 'Test DB');

      // Fill in required auth token field
      const authTokenInput = screen.getByLabelText(/api key/i);
      await user.type(authTokenInput, 'test-token');

      const testButton = screen.getByRole('button', { name: /test connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(screen.getByText(errorMessage)).toBeInTheDocument();
        expect(toast.showToast).toHaveBeenCalledWith(errorMessage, 'error');
      });

      const saveButton = screen.getByRole('button', { name: /save/i });
      expect(saveButton).toBeDisabled();
    });

    it('should validate required fields before testing connection', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      // Clear the baseUrl field to trigger validation
      const baseUrlInput = screen.getByLabelText(/base url/i);
      await user.clear(baseUrlInput);

      const testButton = screen.getByRole('button', { name: /test connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(screen.getByText(/base url is required/i)).toBeInTheDocument();
      });

      expect(api.testChromaConnection).not.toHaveBeenCalled();
    });

    it('should validate basic_auth format requires colon', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const authMethodSelect = screen.getByLabelText(/authentication method/i);
      await user.selectOptions(authMethodSelect, 'basic_auth');

      const authTokenInput = screen.getByLabelText(/api key/i);
      await user.type(authTokenInput, 'usernamepassword'); // Missing colon

      const testButton = screen.getByRole('button', { name: /test connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(screen.getByText(/must be in format 'username:password'/i)).toBeInTheDocument();
      });
    });
  });

  describe('Save Functionality', () => {
    it('should disable save button until connection is tested', () => {
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const saveButton = screen.getByRole('button', { name: /save/i });
      expect(saveButton).toBeDisabled();
    });

    it('should enable save button after successful connection test', async () => {
      const user = userEvent.setup();
      (api.testChromaConnection as jest.Mock).mockResolvedValueOnce({
        success: true,
        message: 'Connection test successful',
      });
      (api.createChromaDatabase as jest.Mock).mockResolvedValueOnce({
        uuid: 'test-uuid',
        message: 'Chroma database connection created successfully',
      });

      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const nameInput = screen.getByLabelText(/connection name/i);
      await user.type(nameInput, 'Test DB');

      // Fill in required auth token field
      const authTokenInput = screen.getByLabelText(/api key/i);
      await user.type(authTokenInput, 'test-token');

      const testButton = screen.getByRole('button', { name: /test connection/i });
      await user.click(testButton);

      await waitFor(() => {
        const saveButton = screen.getByRole('button', { name: /save/i });
        expect(saveButton).not.toBeDisabled();
      });
    });

    it('should save database connection after successful test', async () => {
      const user = userEvent.setup();
      (api.testChromaConnection as jest.Mock).mockResolvedValueOnce({
        success: true,
        message: 'Connection test successful',
      });
      (api.createChromaDatabase as jest.Mock).mockResolvedValueOnce({
        uuid: 'test-uuid',
        message: 'Chroma database connection created successfully',
      });

      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const nameInput = screen.getByLabelText(/connection name/i);
      await user.type(nameInput, 'Test DB');

      // Fill in required auth token field
      const authTokenInput = screen.getByLabelText(/api key/i);
      await user.type(authTokenInput, 'test-token');

      // Test connection first
      const testButton = screen.getByRole('button', { name: /test connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(api.testChromaConnection).toHaveBeenCalled();
      });

      // Save
      const saveButton = screen.getByRole('button', { name: /save/i });
      await user.click(saveButton);

      await waitFor(() => {
        expect(api.createChromaDatabase).toHaveBeenCalled();
        expect(toast.showToast).toHaveBeenCalledWith(
          'Chroma database connection created successfully',
          'success'
        );
        expect(mockOnSuccess).toHaveBeenCalled();
      });
    });

    it('should not save if connection was not tested', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const nameInput = screen.getByLabelText(/connection name/i);
      await user.type(nameInput, 'Test DB');

      // Try to save without testing
      const saveButton = screen.getByRole('button', { name: /save/i });
      expect(saveButton).toBeDisabled();

      // Button is disabled, so clicking it won't do anything
      await user.click(saveButton);

      expect(api.createChromaDatabase).not.toHaveBeenCalled();
    });
  });

  describe('Error Handling', () => {
    it('should display error message in dialog', async () => {
      const user = userEvent.setup();
      const errorMessage = 'Invalid API key';
      (api.testChromaConnection as jest.Mock).mockRejectedValueOnce(new Error(errorMessage));

      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const nameInput = screen.getByLabelText(/connection name/i);
      await user.type(nameInput, 'Test DB');

      // Fill in required auth token field
      const authTokenInput = screen.getByLabelText(/api key/i);
      await user.type(authTokenInput, 'test-token');

      const testButton = screen.getByRole('button', { name: /test connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(screen.getByText(errorMessage)).toBeInTheDocument();
      });
    });

    it('should show error toast when connection test fails', async () => {
      const user = userEvent.setup();
      const errorMessage = 'Connection failed';
      (api.testChromaConnection as jest.Mock).mockRejectedValueOnce(new Error(errorMessage));

      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const nameInput = screen.getByLabelText(/connection name/i);
      await user.type(nameInput, 'Test DB');

      // Fill in required auth token field
      const authTokenInput = screen.getByLabelText(/api key/i);
      await user.type(authTokenInput, 'test-token');

      const testButton = screen.getByRole('button', { name: /test connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(toast.showToast).toHaveBeenCalledWith(errorMessage, 'error');
      });
    });
  });

  describe('Dialog Close Behavior', () => {
    it('should call onClose when close button is clicked', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const closeButton = screen.getByRole('button', { name: /close/i });
      await user.click(closeButton);

      expect(mockOnClose).toHaveBeenCalled();
    });

    it('should call onClose when clicking outside dialog', async () => {
      const user = userEvent.setup();
      const { container } = render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      // Click on the backdrop (the outer div with bg-black/50)
      const backdrop = container.querySelector('.fixed.inset-0');
      if (backdrop) {
        await user.click(backdrop);
        expect(mockOnClose).toHaveBeenCalled();
      }
    });

    it('should call onClose when ESC key is pressed', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      await user.keyboard('{Escape}');

      expect(mockOnClose).toHaveBeenCalled();
    });
  });

  describe('Form Validation', () => {
    it('should validate additional headers format', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const additionalHeadersTextarea = screen.getByLabelText(/additional headers/i);
      await user.type(additionalHeadersTextarea, 'InvalidHeaderFormat');

      // Fill in required fields
      const authTokenInput = screen.getByLabelText(/api key/i);
      await user.type(authTokenInput, 'test-token');

      const testButton = screen.getByRole('button', { name: /test connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(screen.getByText(/Invalid header format/i)).toBeInTheDocument();
      });
    });

    it('should accept valid additional headers format', async () => {
      const user = userEvent.setup();
      (api.testChromaConnection as jest.Mock).mockResolvedValueOnce({
        success: true,
        message: 'Connection test successful',
      });

      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const additionalHeadersTextarea = screen.getByLabelText(/additional headers/i);
      await user.type(additionalHeadersTextarea, 'Custom-Header: value');

      const nameInput = screen.getByLabelText(/connection name/i);
      await user.type(nameInput, 'Test DB');

      // Fill in required auth token field
      const authTokenInput = screen.getByLabelText(/api key/i);
      await user.type(authTokenInput, 'test-token');

      const testButton = screen.getByRole('button', { name: /test connection/i });
      await user.click(testButton);

      await waitFor(() => {
        expect(api.testChromaConnection).toHaveBeenCalled();
      });
    });
  });

  describe('Tooltip', () => {
    it('should show tooltip on disabled save button when connection not tested', async () => {
      const user = userEvent.setup();
      render(
        <AddChromaDatabaseDialog
          isOpen={true}
          onClose={mockOnClose}
          onSuccess={mockOnSuccess}
        />
      );

      const saveButton = screen.getByRole('button', { name: /save/i });
      expect(saveButton).toBeDisabled();

      // Hover over the save button container
      const saveButtonContainer = saveButton.closest('.group');
      if (saveButtonContainer) {
        await user.hover(saveButtonContainer);
        
        await waitFor(() => {
          expect(screen.getByText(/please test the connection first before saving/i)).toBeInTheDocument();
        });
      }
    });
  });
});


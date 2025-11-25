/**
 * Icon Name to Font Awesome Icon Mapper
 * 
 * Maps icon name strings (e.g., "faHome", "home", "Home") to Font Awesome icon definitions.
 */

import { IconDefinition } from '@fortawesome/fontawesome-svg-core';
import {
  faSun,
  faMoon,
  faUser,
  faCog,
  faSignOutAlt,
  faBars,
  faTimes,
  faChevronDown,
  faChevronUp,
  faChevronRight,
  faHome,
  faProjectDiagram,
  faPlay,
  faHistory,
  faEnvelope,
  faServer,
  faDatabase,
  faBook,
  faStore,
  faBuilding,
  faCreditCard,
  faUsers,
  faCloudDownload,
  faPlus,
  faEdit,
  faTrash,
  faCheck,
  faX,
  faInfoCircle,
  faExclamationTriangle,
  faExclamationCircle,
  faCheckCircle,
  faFolder,
  faFile,
  faShare,
  faDownload,
  faTable,
  faFileCode,
  faGithub,
  faTwitter,
  faLinkedin,
} from './icons';

// Create a map of icon names to icon definitions
// Supports both "faIconName" and "iconName" formats
const iconMap = new Map<string, IconDefinition>();

// Helper to add icon with multiple name variations
function addIcon(icon: IconDefinition, baseName: string) {
  const lowerBase = baseName.toLowerCase();
  
  // Add with "fa" prefix: faHome
  iconMap.set(`fa${lowerBase}`, icon);
  
  // Add without "fa" prefix: home
  iconMap.set(lowerBase, icon);
  
  // Add with first letter capitalized: Home
  iconMap.set(lowerBase.charAt(0).toUpperCase() + lowerBase.slice(1), icon);
  
  // Add with hyphens: fa-home, home
  const hyphenated = lowerBase.replace(/([A-Z])/g, '-$1').toLowerCase();
  iconMap.set(`fa-${hyphenated}`, icon);
  iconMap.set(hyphenated, icon);
}

// Add all icons to the map
addIcon(faSun, 'Sun');
addIcon(faMoon, 'Moon');
addIcon(faUser, 'User');
addIcon(faCog, 'Cog');
addIcon(faSignOutAlt, 'SignOutAlt');
addIcon(faBars, 'Bars');
addIcon(faTimes, 'Times');
addIcon(faChevronDown, 'ChevronDown');
addIcon(faChevronUp, 'ChevronUp');
addIcon(faChevronRight, 'ChevronRight');
addIcon(faHome, 'Home');
addIcon(faProjectDiagram, 'ProjectDiagram');
addIcon(faPlay, 'Play');
addIcon(faHistory, 'History');
addIcon(faEnvelope, 'Envelope');
addIcon(faServer, 'Server');
addIcon(faDatabase, 'Database');
addIcon(faBook, 'Book');
addIcon(faStore, 'Store');
addIcon(faBuilding, 'Building');
addIcon(faCreditCard, 'CreditCard');
addIcon(faUsers, 'Users');
addIcon(faCloudDownload, 'CloudDownload');
addIcon(faPlus, 'Plus');
addIcon(faEdit, 'Edit');
addIcon(faTrash, 'Trash');
addIcon(faCheck, 'Check');
addIcon(faX, 'X');
addIcon(faInfoCircle, 'InfoCircle');
addIcon(faExclamationTriangle, 'ExclamationTriangle');
addIcon(faExclamationCircle, 'ExclamationCircle');
addIcon(faCheckCircle, 'CheckCircle');
addIcon(faFolder, 'Folder');
addIcon(faFile, 'File');
addIcon(faShare, 'Share');
addIcon(faDownload, 'Download');
addIcon(faTable, 'Table');
addIcon(faFileCode, 'FileCode');
addIcon(faGithub, 'Github');
addIcon(faTwitter, 'Twitter');
addIcon(faLinkedin, 'Linkedin');

/**
 * Get a Font Awesome icon by name
 * 
 * @param iconName - Icon name (e.g., "faHome", "home", "Home", "fa-user", "user")
 * @returns IconDefinition if found, null otherwise
 */
export function getIconByName(iconName: string | null | undefined): IconDefinition | null {
  if (!iconName) return null;
  
  // Normalize the icon name
  const normalized = iconName
    .trim()
    .toLowerCase()
    .replace(/^fa-/, '') // Remove "fa-" prefix if present
    .replace(/-/g, ''); // Remove hyphens
  
  // Try exact match first
  let icon = iconMap.get(normalized);
  if (icon) return icon;
  
  // Try with "fa" prefix
  icon = iconMap.get(`fa${normalized}`);
  if (icon) return icon;
  
  // Try capitalized version
  const capitalized = normalized.charAt(0).toUpperCase() + normalized.slice(1);
  icon = iconMap.get(capitalized);
  if (icon) return icon;
  
  return null;
}

/**
 * Get icon name suggestions for autocomplete/validation
 */
export function getAvailableIconNames(): string[] {
  return Array.from(iconMap.keys()).sort();
}


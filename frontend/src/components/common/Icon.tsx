"use client";

import { FontAwesomeIcon, FontAwesomeIconProps } from '@fortawesome/react-fontawesome';
import { IconDefinition } from '@fortawesome/fontawesome-svg-core';

interface IconProps extends Omit<FontAwesomeIconProps, 'icon'> {
  icon: IconDefinition;
}

/**
 * Icon component wrapper for Font Awesome icons
 * 
 * Usage:
 * ```tsx
 * import { Icon } from '@/components/common/Icon';
 * import { faHome } from '@/lib/icons';
 * 
 * <Icon icon={faHome} size="lg" className="text-blue-500" />
 * ```
 */
export function Icon({ 
  icon, 
  className = '', 
  size = '1x',
  ...props 
}: IconProps) {
  return (
    <FontAwesomeIcon 
      icon={icon} 
      className={className}
      size={size}
      {...props}
    />
  );
}


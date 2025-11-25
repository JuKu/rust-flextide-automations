# Font Awesome Icons Integration

This project uses **Font Awesome Free** icons for consistent, high-quality iconography throughout the UI.

## License Compatibility ✅

Font Awesome Free is fully compatible with this project's license (MIT/Apache-2.0):

- **Icons**: CC BY 4.0 (Creative Commons Attribution 4.0)
- **Fonts**: SIL OFL 1.1 (SIL Open Font License)
- **Code**: MIT License

## Usage

### Basic Usage

```tsx
import { Icon } from '@/components/common/Icon';
import { faHome, faUser, faCog } from '@/lib/icons';

// Simple icon
<Icon icon={faHome} />

// With size and styling
<Icon icon={faUser} size="lg" className="text-blue-500" />

// With additional props
<Icon 
  icon={faCog} 
  size="2x" 
  spin 
  className="text-gray-600"
/>
```

### Available Icons

Common icons are pre-exported in `@/lib/icons.ts`. To add more:

1. Import from `@fortawesome/free-solid-svg-icons` (or `free-regular-svg-icons`, `free-brands-svg-icons`)
2. Add to the `library.add()` call in `icons.ts`
3. Export the icon

Example:
```ts
import { faNewIcon } from '@fortawesome/free-solid-svg-icons';
library.add(faNewIcon);
export { faNewIcon };
```

### Icon Sets

- **Solid Icons** (`free-solid-svg-icons`): Most common, filled icons
- **Regular Icons** (`free-regular-svg-icons`): Outlined icons
- **Brand Icons** (`free-brands-svg-icons`): Social media and brand logos

## Attribution

Font Awesome Free requires attribution. Add this to your project:

**Icons by [Font Awesome](https://fontawesome.com/)**

This can be added to:
- Footer component
- About page
- README.md
- License file

## Tree Shaking

Icons are tree-shaken automatically - only imported icons are included in the bundle.

## Resources

- [Font Awesome Icons](https://fontawesome.com/icons)
- [React Font Awesome Docs](https://fontawesome.com/docs/web/use-with/react/)
- [License Information](https://fontawesome.com/license/free)


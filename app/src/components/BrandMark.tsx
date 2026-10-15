export function BrandMark() {
  return (
    <svg className="brand-mark" viewBox="0 0 36 36" aria-hidden="true">
      <path className="brand-mark-frame" d="M4 8.5 17.5 4v25L4 25.5zM32 8.5 18.5 4v25L32 25.5z" />
      <path
        className="brand-mark-slot"
        d="M7 10.5 17.5 7v18.5L7 22.5v-12Zm22 0L18.5 7v18.5L29 22.5v-12ZM18 7v18.5"
      />
      <path
        className="brand-mark-slot"
        d="M10 13.5 14.5 12M10 17 14.5 15.5M21.5 12 26 13.5M21.5 15.5 26 17"
      />
      <circle className="brand-mark-node" cx="18" cy="7" r="1.7" />
    </svg>
  );
}

export function AngularButton({
  href,
  children,
  secondary = false,
}: {
  href: string;
  children: React.ReactNode;
  secondary?: boolean;
}) {
  return (
    <a href={href} className={`angular-button ${secondary ? "secondary" : ""}`}>
      {children}
      <span>↗</span>
    </a>
  );
}

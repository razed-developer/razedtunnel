export type ThemePreference = "system" | "light" | "dark";

export default function ThemePicker({ value, onChange }: { value: ThemePreference; onChange: (theme: ThemePreference) => void }) {
  return <label className="theme-picker"><span>Appearance</span><select aria-label="Appearance theme" value={value} onChange={(event) => onChange(event.target.value as ThemePreference)}><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></label>;
}

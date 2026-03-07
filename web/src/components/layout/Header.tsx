import { useLocation } from 'react-router-dom';
import { LogOut, Globe } from 'lucide-react';
import { t, getSupportedLocales, type Locale } from '@/lib/i18n';
import { useLocaleContext } from '@/App';
import { useAuth } from '@/hooks/useAuth';

const routeTitles: Record<string, string> = {
  '/': 'nav.dashboard',
  '/agent': 'nav.agent',
  '/tools': 'nav.tools',
  '/cron': 'nav.cron',
  '/integrations': 'nav.integrations',
  '/memory': 'nav.memory',
  '/config': 'nav.config',
  '/cost': 'nav.cost',
  '/logs': 'nav.logs',
  '/doctor': 'nav.doctor',
};

const localeLabels: Record<Locale, string> = {
  en: 'English',
  'zh-CN': '简体中文',
  tr: 'Türkçe',
};

export default function Header() {
  const location = useLocation();
  const { logout } = useAuth();
  const { locale, setAppLocale } = useLocaleContext();

  const titleKey = routeTitles[location.pathname] ?? 'nav.dashboard';
  const pageTitle = t(titleKey);

  const handleLocaleChange = (newLocale: Locale) => {
    setAppLocale(newLocale);
  };

  return (
    <header className="h-14 bg-gray-800 border-b border-gray-700 flex items-center justify-between px-6">
      <h1 className="text-lg font-semibold text-white">{pageTitle}</h1>

      <div className="flex items-center gap-4">
        <div className="relative">
          <div className="flex items-center gap-1 px-3 py-1 rounded-md text-sm font-medium border border-gray-600 text-gray-300 hover:bg-gray-700 hover:text-white transition-colors cursor-pointer">
            <Globe className="h-4 w-4" />
            <select
              value={locale}
              onChange={(e) => handleLocaleChange(e.target.value as Locale)}
              className="appearance-none bg-transparent cursor-pointer focus:outline-none"
            >
              {getSupportedLocales().map((loc) => (
                <option key={loc} value={loc} className="bg-gray-800">
                  {localeLabels[loc]}
                </option>
              ))}
            </select>
          </div>
        </div>

        <button
          type="button"
          onClick={logout}
          className="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm text-gray-300 hover:bg-gray-700 hover:text-white transition-colors"
        >
          <LogOut className="h-4 w-4" />
          <span>{t('auth.logout')}</span>
        </button>
      </div>
    </header>
  );
}

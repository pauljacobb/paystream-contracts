import { useTranslation } from 'react-i18next';

export default function LanguageSwitcher() {
  const { i18n } = useTranslation();

  function handleChange(e) {
    const lang = e.target.value;
    i18n.changeLanguage(lang);
    document.documentElement.dir = lang === 'ar' ? 'rtl' : 'ltr';
    document.documentElement.lang = lang;
  }

  return (
    <select value={i18n.language} onChange={handleChange} aria-label="Select language">
      <option value="en">English</option>
      <option value="ar">العربية</option>
    </select>
  );
}

import Header from "@/components/Header";
import Sidebar from "@/components/Sidebar";
import Footer from "@/components/Footer";

export default function DocsLayout({ children }: { children: React.ReactNode }) {
  return (
    <>
      <Header />
      <div className="docs-wrapper">
        <Sidebar />
        <main className="docs-main">
          <div className="docs-content">{children}</div>
          <Footer />
        </main>
      </div>
    </>
  );
}

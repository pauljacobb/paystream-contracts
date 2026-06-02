import { useRef } from 'react';
import { BarChart, Bar, XAxis, YAxis, Tooltip, ResponsiveContainer } from 'recharts';

// TODO: replace mock data with real RPC calls to the stream contract
const MOCK_STATS = {
  totalLocked: '42500.00',
  activeStreams: 12,
  pausedStreams: 3,
};

const MOCK_MONTHLY_SPEND = [
  { month: 'Jan', spend: 6800 },
  { month: 'Feb', spend: 7200 },
  { month: 'Mar', spend: 7100 },
  { month: 'Apr', spend: 6950 },
  { month: 'May', spend: 7400 },
  { month: 'Jun', spend: 7050 },
];

export default function EmployerDashboard() {
  const chartRef = useRef(null);

  async function exportToPng() {
    const { default: html2canvas } = await import('html2canvas');
    const canvas = await html2canvas(chartRef.current);
    const link = document.createElement('a');
    link.download = 'monthly-spend.png';
    link.href = canvas.toDataURL('image/png');
    link.click();
  }

  return (
    <div>
      <h2>Employer Dashboard</h2>

      <div>
        <div>
          <h3>Total Funds Locked</h3>
          <p>{MOCK_STATS.totalLocked} USDC</p>
        </div>
        <div>
          <h3>Active Streams</h3>
          <p>{MOCK_STATS.activeStreams}</p>
        </div>
        <div>
          <h3>Paused Streams</h3>
          <p>{MOCK_STATS.pausedStreams}</p>
        </div>
      </div>

      <div ref={chartRef}>
        <h3>Monthly Spend — Last 6 Months</h3>
        <ResponsiveContainer width="100%" height={300}>
          <BarChart data={MOCK_MONTHLY_SPEND}>
            <XAxis dataKey="month" />
            <YAxis />
            <Tooltip />
            <Bar dataKey="spend" />
          </BarChart>
        </ResponsiveContainer>
      </div>

      <button onClick={exportToPng}>Export Chart to PNG</button>
    </div>
  );
}

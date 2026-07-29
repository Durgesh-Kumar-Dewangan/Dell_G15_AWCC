'use client';

import {
  LineChart,
  Line,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from 'recharts';

interface ChartDataPoint {
  time: string;
  cpu?: number;
  gpu?: number;
}

interface TemperatureChartProps {
  data: ChartDataPoint[];
  title: string;
  type?: 'line' | 'area';
}

export function TemperatureChart({
  data,
  title,
  type = 'area',
}: TemperatureChartProps) {
  const ChartComponent = type === 'area' ? AreaChart : LineChart;
  const DataComponent = type === 'area' ? Area : Line;

  return (
    <div className="glass rounded-lg p-6 border border-border-light h-80">
      <h3 className="text-lg font-semibold text-text mb-4">{title}</h3>
      <ResponsiveContainer width="100%" height="100%" minHeight={300}>
        <ChartComponent data={data} margin={{ top: 5, right: 30, left: 0, bottom: 5 }}>
          <defs>
            <linearGradient id="cpuGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#0077ff" stopOpacity={0.3} />
              <stop offset="95%" stopColor="#0077ff" stopOpacity={0} />
            </linearGradient>
            <linearGradient id="gpuGradient" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="#ff8c42" stopOpacity={0.3} />
              <stop offset="95%" stopColor="#ff8c42" stopOpacity={0} />
            </linearGradient>
          </defs>
          <CartesianGrid strokeDasharray="3 3" stroke="#383838" />
          <XAxis
            dataKey="time"
            stroke="#707070"
            style={{ fontSize: '12px' }}
          />
          <YAxis
            stroke="#707070"
            style={{ fontSize: '12px' }}
            label={{ value: '°C', angle: -90, position: 'insideLeft' }}
          />
          <Tooltip
            contentStyle={{
              backgroundColor: '#1a1a1a',
              border: '1px solid #383838',
              borderRadius: '8px',
            }}
            cursor={{ stroke: '#383838' }}
          />
          {type === 'area' ? (
            <>
              {data[0]?.cpu !== undefined && (
                <Area
                  type="monotone"
                  dataKey="cpu"
                  stroke="#0077ff"
                  fillOpacity={1}
                  fill="url(#cpuGradient)"
                  strokeWidth={2}
                  name="CPU"
                  isAnimationActive={false}
                />
              )}
              {data[0]?.gpu !== undefined && (
                <Area
                  type="monotone"
                  dataKey="gpu"
                  stroke="#ff8c42"
                  fillOpacity={1}
                  fill="url(#gpuGradient)"
                  strokeWidth={2}
                  name="GPU"
                  isAnimationActive={false}
                />
              )}
            </>
          ) : (
            <>
              {data[0]?.cpu !== undefined && (
                <Line
                  type="monotone"
                  dataKey="cpu"
                  stroke="#0077ff"
                  strokeWidth={2}
                  dot={false}
                  name="CPU"
                  isAnimationActive={false}
                />
              )}
              {data[0]?.gpu !== undefined && (
                <Line
                  type="monotone"
                  dataKey="gpu"
                  stroke="#ff8c42"
                  strokeWidth={2}
                  dot={false}
                  name="GPU"
                  isAnimationActive={false}
                />
              )}
            </>
          )}
        </ChartComponent>
      </ResponsiveContainer>
    </div>
  );
}

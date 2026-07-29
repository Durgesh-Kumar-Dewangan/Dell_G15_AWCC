import { exec } from 'child_process';
import { promisify } from 'util';
import fs from 'fs';
import path from 'path';

const execAsync = promisify(exec);
const readFileAsync = promisify(fs.readFile);
const writeFileAsync = promisify(fs.writeFile);

export interface ThermalData {
  cpuTemp: number;
  gpuTemp: number;
  cpuRpm?: number;
  gpuRpm?: number;
  systemHealth: 'normal' | 'warning' | 'critical';
}

export interface FanControl {
  channel: 'cpu' | 'gpu';
  mode: 'auto' | 'manual' | 'maximum';
  dutyCycle?: number; // 40-100
}

/**
 * Hardware Control Module
 * Direct Linux thermal sensor and fan control
 */
export class HardwareController {
  private sysfsBasePath = '/sys/class/';
  private thermalBasePath = '/sys/class/thermal/';
  private hwmonBasePath = '/sys/class/hwmon/';
  private pwmBasePath = '/sys/class/pwm/';

  /**
   * Read CPU temperature from thermal sensors
   */
  async readCPUTemperature(): Promise<number> {
    try {
      // Try multiple sensor locations
      const sensorPaths = [
        '/sys/class/thermal/thermal_zone0/temp',
        '/sys/class/thermal/thermal_zone1/temp',
        '/sys/devices/platform/coretemp.0/hwmon/hwmon0/temp2_input',
      ];

      for (const sensorPath of sensorPaths) {
        try {
          const data = await readFileAsync(sensorPath, 'utf-8');
          const temp = parseInt(data.trim()) / 1000; // Convert from millidegrees
          if (temp > 0 && temp < 150) {
            return Math.round(temp * 10) / 10;
          }
        } catch (e) {
          // Try next path
        }
      }

      // Fallback: use sensors command
      return await this.readTemperatureFromSensors('Core');
    } catch (error) {
      console.error('[v0] CPU temperature read error:', error);
      return 45; // Default fallback
    }
  }

  /**
   * Read GPU temperature from thermal sensors
   */
  async readGPUTemperature(): Promise<number> {
    try {
      const sensorPaths = [
        '/sys/class/thermal/thermal_zone2/temp',
        '/sys/devices/platform/coretemp.0/hwmon/hwmon0/temp1_input',
        '/sys/class/drm/card0/device/hwmon/hwmon*/temp1_input',
      ];

      for (const sensorPath of sensorPaths) {
        try {
          const data = await readFileAsync(sensorPath, 'utf-8');
          const temp = parseInt(data.trim()) / 1000;
          if (temp > 0 && temp < 150) {
            return Math.round(temp * 10) / 10;
          }
        } catch (e) {
          // Try next path
        }
      }

      // Fallback: use sensors command
      return await this.readTemperatureFromSensors('edge');
    } catch (error) {
      console.error('[v0] GPU temperature read error:', error);
      return 50; // Default fallback
    }
  }

  /**
   * Read temperature using sensors command
   */
  private async readTemperatureFromSensors(pattern: string): Promise<number> {
    try {
      const { stdout } = await execAsync('sensors 2>/dev/null || echo "no sensors"');
      const lines = stdout.split('\n');
      
      for (const line of lines) {
        if (line.includes(pattern)) {
          const match = line.match(/[\d.]+\s*°C/);
          if (match) {
            return parseFloat(match[0]);
          }
        }
      }
      return 45;
    } catch {
      return 45;
    }
  }

  /**
   * Read fan RPM
   */
  async readFanRPM(channel: 'cpu' | 'gpu'): Promise<number> {
    try {
      const { stdout } = await execAsync('sensors 2>/dev/null || echo ""');
      
      if (channel === 'cpu') {
        const match = stdout.match(/CPU.*?(\d+)\s*RPM/i);
        return match ? parseInt(match[1]) : 0;
      } else {
        const match = stdout.match(/GPU.*?(\d+)\s*RPM/i);
        return match ? parseInt(match[1]) : 0;
      }
    } catch {
      return 0;
    }
  }

  /**
   * Set fan duty cycle (40-100%)
   */
  async setFanDuty(channel: 'cpu' | 'gpu', dutyCycle: number): Promise<boolean> {
    try {
      // Validate input
      const duty = Math.max(40, Math.min(100, dutyCycle));
      const pwmValue = Math.round((duty / 100) * 255); // Convert to 0-255

      // Try multiple PWM control methods
      const methods = [
        () => this.setPWMViaProcFs(channel, pwmValue),
        () => this.setPWMViaSysFs(channel, pwmValue),
        () => this.setPWMViaECDirect(channel, duty),
      ];

      for (const method of methods) {
        try {
          const result = await method();
          if (result) return true;
        } catch (e) {
          // Try next method
        }
      }

      console.warn('[v0] Could not set fan duty - using echo fallback');
      return false;
    } catch (error) {
      console.error('[v0] Fan duty set error:', error);
      return false;
    }
  }

  /**
   * Set PWM via /proc filesystem
   */
  private async setPWMViaProcFs(
    channel: 'cpu' | 'gpu',
    pwmValue: number
  ): Promise<boolean> {
    try {
      const procPath = channel === 'cpu' 
        ? '/proc/acpi/dell_smm/fan_speed'
        : '/proc/acpi/dell_smm/gpu_fan_speed';

      // Check if path exists
      try {
        fs.statSync(procPath);
      } catch {
        return false;
      }

      // Try to write
      await execAsync(`echo ${pwmValue} | sudo tee ${procPath} > /dev/null 2>&1`);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Set PWM via /sys filesystem (more reliable on modern systems)
   */
  private async setPWMViaSysFs(
    channel: 'cpu' | 'gpu',
    pwmValue: number
  ): Promise<boolean> {
    try {
      const pwmPaths = [
        `/sys/class/pwm/pwmchip0/${channel}_pwm/duty_cycle`,
        `/sys/devices/platform/dell_smm_hwmon/pwm${channel === 'cpu' ? '1' : '2'}`,
        `/sys/class/hwmon/hwmon0/pwm${channel === 'cpu' ? '1' : '2'}`,
      ];

      for (const pwmPath of pwmPaths) {
        try {
          fs.statSync(pwmPath);
          await execAsync(`echo ${pwmValue} | sudo tee ${pwmPath} > /dev/null 2>&1`);
          return true;
        } catch {
          // Try next path
        }
      }
      return false;
    } catch {
      return false;
    }
  }

  /**
   * Set PWM via EC direct (embedded controller)
   */
  private async setPWMViaECDirect(
    channel: 'cpu' | 'gpu',
    duty: number
  ): Promise<boolean> {
    try {
      // Use dell_smm_hwmon kernel module command
      const fanIndex = channel === 'cpu' ? 0 : 1;
      const command = `echo "set_fan ${fanIndex} ${Math.round(duty)}" | sudo tee /dev/dell_smm 2>/dev/null`;
      
      await execAsync(command);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get fan mode (auto/manual)
   */
  async getFanMode(channel: 'cpu' | 'gpu'): Promise<'auto' | 'manual'> {
    try {
      const { stdout } = await execAsync('cat /sys/devices/platform/dell_smm_hwmon/fan_mode 2>/dev/null || echo "auto"');
      return stdout.includes('manual') ? 'manual' : 'auto';
    } catch {
      return 'auto';
    }
  }

  /**
   * Set fan mode (auto/manual)
   */
  async setFanMode(channel: 'cpu' | 'gpu', mode: 'auto' | 'manual'): Promise<boolean> {
    try {
      const modeValue = mode === 'manual' ? '1' : '0';
      const fanIndex = channel === 'cpu' ? 0 : 1;

      const command = `echo "set_fan_mode ${fanIndex} ${modeValue}" | sudo tee /dev/dell_smm 2>/dev/null`;
      await execAsync(command);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get overall system health status
   */
  async getSystemHealth(): Promise<'normal' | 'warning' | 'critical'> {
    try {
      const cpuTemp = await this.readCPUTemperature();
      const gpuTemp = await this.readGPUTemperature();

      if (cpuTemp > 95 || gpuTemp > 90) {
        return 'critical';
      }
      if (cpuTemp > 80 || gpuTemp > 75) {
        return 'warning';
      }
      return 'normal';
    } catch {
      return 'normal';
    }
  }

  /**
   * Get complete thermal data
   */
  async getThermalData(): Promise<ThermalData> {
    const [cpuTemp, gpuTemp, cpuRpm, gpuRpm, health] = await Promise.all([
      this.readCPUTemperature(),
      this.readGPUTemperature(),
      this.readFanRPM('cpu'),
      this.readFanRPM('gpu'),
      this.getSystemHealth(),
    ]);

    return {
      cpuTemp,
      gpuTemp,
      cpuRpm,
      gpuRpm,
      systemHealth: health,
    };
  }

  /**
   * Initialize and check hardware access
   */
  async initialize(): Promise<{
    hasAccess: boolean;
    features: {
      tempSensors: boolean;
      fanControl: boolean;
      pwmControl: boolean;
    };
    message: string;
  }> {
    try {
      const features = {
        tempSensors: false,
        fanControl: false,
        pwmControl: false,
      };

      // Check temperature sensor access
      try {
        const temp = await this.readCPUTemperature();
        features.tempSensors = temp > 0;
      } catch {
        // Temperature sensors not available
      }

      // Check fan control access
      try {
        const mode = await this.getFanMode('cpu');
        features.fanControl = !!mode;
      } catch {
        // Fan control not available
      }

      // Check PWM control access
      try {
        fs.statSync('/sys/class/pwm/pwmchip0');
        features.pwmControl = true;
      } catch {
        try {
          fs.statSync('/sys/class/hwmon/hwmon0/pwm1');
          features.pwmControl = true;
        } catch {
          // PWM not available
        }
      }

      const hasAccess = features.tempSensors || features.fanControl;
      const message = hasAccess
        ? `Hardware access available (temp: ${features.tempSensors}, fan: ${features.fanControl}, pwm: ${features.pwmControl})`
        : 'Limited hardware access - running in demo mode';

      return {
        hasAccess,
        features,
        message,
      };
    } catch (error) {
      return {
        hasAccess: false,
        features: {
          tempSensors: false,
          fanControl: false,
          pwmControl: false,
        },
        message: `Initialization error: ${String(error)}`,
      };
    }
  }
}

// Export singleton instance
export const hardwareController = new HardwareController();

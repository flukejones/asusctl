declare const global: any, imports: any;
declare var asusctlGexInstance: any;
//@ts-ignore
const Me = imports.misc.extensionUtils.getCurrentExtension();

import { DbusBase } from '../modules/dbus';

// function getMethods(obj: { [x: string]: { toString: () => string; }; }) {
//     var result = [];
//     for (var id in obj) {
//       try {
//         if (typeof(obj[id]) == "function") {
//           result.push(id + ": " + obj[id].toString());
//         }
//       } catch (err) {
//         result.push(id + ": inaccessible");
//       }
//     }
//     return result;
//   }

export class ChargingLimit extends DbusBase {
    lastState: number = 100;

    constructor() {
        super('org-asuslinux-power-4', '/org/asuslinux/Power');
    }

    public getChargingLimit() {
        if (this.isRunning()) {
            try {
                this.lastState = this.dbus_proxy.ChargeControlEndThresholdSync();
            } catch (e) {
                //@ts-ignore
                log(`Failed to fetch Charging Limit!`, e);
            }
        }
        return this.lastState;
    }

    public setChargingLimit(limit: number) {
        if (this.isRunning()) {
            try {
                if (limit > 0 && this.lastState !== limit) {
                    // update state
                    this.lastState = limit;
                }
                return this.dbus_proxy.SetChargeControlEndThresholdSync(limit);
            } catch (e) {
                //@ts-ignore
                log(`Profile DBus set power profile failed!`, e);
            }
        }
    }

    async start() {
        try {
            await super.start();
            this.getChargingLimit();

            this.dbus_proxy.connectSignal(
                "NotifyChargeControlEndThreshold",
                (proxy: any = null, name: string, data: string) => {
                    if (proxy) {
                        //@ts-ignore
                        log(`Charging Limit has changed to ${data}% (${name}).`);
                        this.lastState = parseInt(data);
                    }
                }
            );
        } catch (e) {
            //@ts-ignore
            log(`Charging Limit DBus initialization failed!`, e);
        }
    }

    async stop() {
        await super.stop();
    }
}
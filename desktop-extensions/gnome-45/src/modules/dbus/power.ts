import { DbusBase } from "./base";

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

export class Power extends DbusBase {
    chargeLimit = 100;
    mainsOnline = false;

    constructor() {
        super("org-asuslinux-power-4", "/org/asuslinux/Power");
    }

    public getChargingLimit() {
        if (this.isRunning()) {
            try {
                this.chargeLimit = this.dbus_proxy.ChargeControlEndThresholdSync();
            } catch (e) {
                //@ts-ignore
                log("Failed to fetch Charging Limit!", e);
            }
        }
        return this.chargeLimit;
    }

    public setChargingLimit(limit: number) {
        if (this.isRunning()) {
            try {
                if (limit > 0 && this.chargeLimit !== limit) {
                    // update state
                    this.chargeLimit = limit;
                }
                return this.dbus_proxy.SetChargeControlEndThresholdSync(limit);
            } catch (e) {
                //@ts-ignore
                log("Profile DBus set power profile failed!", e);
            }
        }
    }

    public getMainsOnline() {
        if (this.isRunning()) {
            try {
                this.mainsOnline = this.dbus_proxy.MainsOnlineSync();
            } catch (e) {
                //@ts-ignore
                log("Failed to fetch MainsLonline!", e);
            }
        }
        return this.mainsOnline;
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
                        this.chargeLimit = parseInt(data);
                    }
                }
            );

            this.dbus_proxy.connectSignal(
                "NotifyMainsOnline",
                (proxy: any = null, name: string, data: string) => {
                    if (proxy) {
                        //@ts-ignore
                        log(`NotifyMainsOnline has changed to ${data}% (${name}).`);
                        this.mainsOnline = parseInt(data) == 1 ? true : false;
                    }
                }
            );
        } catch (e) {
            //@ts-ignore
            log("Charging Limit DBus initialization failed!", e);
        }
    }

    async stop() {
        await super.stop();
    }
}
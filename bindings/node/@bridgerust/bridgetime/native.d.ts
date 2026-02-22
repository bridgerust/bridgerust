/* eslint-disable */
/* auto-generated-like local declaration for BridgeTime bindings */

export declare class BridgeTime {
  constructor(input?: string | null, timezone?: string | null)

  static now(timezone?: string | null): BridgeTime
  static parse(input: string, timezone?: string | null): BridgeTime
  static fromUnixMs(unixMs: bigint | number, timezone?: string | null): BridgeTime
  static fromUnix(unixSeconds: bigint | number, timezone?: string | null): BridgeTime

  toIso(): string
  format(pattern: string): string
  unixMs(): bigint
  unix(): bigint
  valueOf(): bigint
  timezone(): string
  toTimezone(timezone: string): BridgeTime
  add(amount: bigint | number, unit: string): BridgeTime
  subtract(amount: bigint | number, unit: string): BridgeTime
  startOf(unit: string): BridgeTime
  endOf(unit: string): BridgeTime
  diff(other: BridgeTime, unit: string, asFloat?: boolean | null): number
  isValid(): boolean
  daysInMonth(): number
  isLeapYear(): boolean
  get(field: string): bigint
  set(field: string, value: bigint | number): BridgeTime
  isBefore(other: BridgeTime): boolean
  isAfter(other: BridgeTime): boolean
  isSame(other: BridgeTime): boolean
  isBeforeUnit(other: BridgeTime, unit: string): boolean
  isAfterUnit(other: BridgeTime, unit: string): boolean
  isSameUnit(other: BridgeTime, unit: string): boolean
  isSameOrBefore(other: BridgeTime): boolean
  isSameOrAfter(other: BridgeTime): boolean
  isBetween(
    start: BridgeTime,
    end: BridgeTime,
    unit?: string | null,
    inclusivity?: string | null
  ): boolean
  cloneTime(): BridgeTime
}

export declare function supportedUnits(): Array<string>

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
  year(): bigint
  month(): bigint
  date(): bigint
  day(): bigint
  hour(): bigint
  minute(): bigint
  second(): bigint
  millisecond(): bigint
  setYear(value: bigint | number): BridgeTime
  setMonth(value: bigint | number): BridgeTime
  setDate(value: bigint | number): BridgeTime
  setDay(value: bigint | number): BridgeTime
  setHour(value: bigint | number): BridgeTime
  setMinute(value: bigint | number): BridgeTime
  setSecond(value: bigint | number): BridgeTime
  setMillisecond(value: bigint | number): BridgeTime
  dayOfYear(): number
  setDayOfYear(value: bigint | number): BridgeTime
  quarter(): number
  setQuarter(value: bigint | number): BridgeTime
  isoWeekday(): number
  setIsoWeekday(value: bigint | number): BridgeTime
  isoWeek(): number
  setIsoWeek(value: bigint | number): BridgeTime
  weekOfYear(): number
  week(): number
  setWeek(value: bigint | number): BridgeTime
  isToday(): boolean
  isYesterday(): boolean
  isTomorrow(): boolean
  fromTime(other: BridgeTime, withoutSuffix?: boolean | null): string
  toTime(other: BridgeTime, withoutSuffix?: boolean | null): string
  fromNow(withoutSuffix?: boolean | null): string
  toNow(withoutSuffix?: boolean | null): string
  isBefore(other: BridgeTime): boolean
  isAfter(other: BridgeTime): boolean
  isSame(other: BridgeTime): boolean
  isBeforeUnit(other: BridgeTime, unit: string): boolean
  isAfterUnit(other: BridgeTime, unit: string): boolean
  isSameUnit(other: BridgeTime, unit: string): boolean
  isSameOrBefore(other: BridgeTime): boolean
  isSameOrAfter(other: BridgeTime): boolean
  isSameOrBeforeUnit(other: BridgeTime, unit: string): boolean
  isSameOrAfterUnit(other: BridgeTime, unit: string): boolean
  isBetween(
    start: BridgeTime,
    end: BridgeTime,
    unit?: string | null,
    inclusivity?: string | null
  ): boolean
  cloneTime(): BridgeTime
}

export declare function supportedUnits(): Array<string>

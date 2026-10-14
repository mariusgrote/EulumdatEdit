// Lamp-set count bounds. These mirror the EULUMDAT validation contract enforced
// by the locked eulumdat-core dependency (`1..=20` in validation.rs); update both
// together. The backend stays the final validator; these only keep UI actions
// from producing a document it would reject.
export const MIN_LAMP_SETS = 1;
export const MAX_LAMP_SETS = 20;

export const canAddLampSet = (count: number, absolute = false) =>
  !absolute && count < MAX_LAMP_SETS;
export const canRemoveLampSet = (count: number) => count > MIN_LAMP_SETS;

export const canUseAbsolutePhotometry = (count: number) => count === 1;

/** Converts the positive count shown by the editor back to the file marker. */
export const storedLampCount = (count: number, absolute: boolean) =>
  absolute ? -Math.abs(count) : Math.abs(count);

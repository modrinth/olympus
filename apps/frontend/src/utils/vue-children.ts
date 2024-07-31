import { createTextVNode, isVNode, toDisplayString, type VNode } from "vue";

/**
 * Checks whether a specific child is a VNode. If not, converts it to a display
 * string and then creates text VNode for the result.
 *
 * @param child Child to normalize.
 * @returns Either the original VNode or a text VNode containing child converted
 * to a display string.
 */
function normalizeChild(child: unknown): VNode {
  return isVNode(child) ? child : createTextVNode(toDisplayString(child));
}

/**
 * Takes in an array of VNodes and other children. It then converts each child
 * that is not already a VNode to a display string, and creates a text VNode for
 * that string.
 *
 * @param children Children to normalize.
 * @returns Children with all of non-VNodes converted to display strings.
 */
export function normalizeChildren(children: unknown | unknown[]): VNode[] {
  return Array.isArray(children) ? children.map(normalizeChild) : [normalizeChild(children)];
}

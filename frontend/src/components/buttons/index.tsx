import React, {
	type ComponentProps,
	useEffect,
	useState,
	useMemo,
	type ComponentType,
	type AnchorHTMLAttributes,
} from "react";
import { createLink } from "@tanstack/react-router";
import clsx from "clsx";
import { PlusIcon } from "@storybook/icons";

import styles from "./buttons.module.css";

type IconProps = {
	Icon: ComponentType<ComponentProps<typeof PlusIcon>>;
	iconLabel: string;
	size?: "big" | "small";
};

type BtnProps = {
	animate?: boolean;
	error?: boolean;
};

type OmitProps = "style" | "aria-label";

type CustomButtonProps<T> = T & IconProps & BtnProps;
type PartialCustomButtonProps<T> = T & Partial<IconProps> & Partial<BtnProps>;

function useLinkAnimation(shouldAnimate: boolean = false) {
	const [showBorder, setShowBorder] = useState(shouldAnimate);
	const [animating, setAnimating] = useState(shouldAnimate);

	useEffect(() => {
		let timeout = [];
		if (shouldAnimate) {
			timeout.push(
				setTimeout(() => {
					setAnimating(true);
					setShowBorder(true);
				}, 0),
			);
		} else {
			timeout.push(
				setTimeout(() => {
					setShowBorder(false);
				}, 0),
			);
			timeout.push(
				setTimeout(() => {
					setAnimating(false);
				}, 500),
			);
		}

		return () => timeout.forEach((t) => clearTimeout(t));
	}, [shouldAnimate]);

	return { showBorder, animating };
}

function useButtonProps<T>(props: CustomButtonProps<T>) {
	return useMemo(() => {
		let copied: PartialCustomButtonProps<T> = { ...props };

		let icon = {
			Icon: props.Icon,
			iconLabel: props.iconLabel,
			size: props.size,
		};
		let btn = {
			animate: props.animate,
			error: props.error,
		};

		delete copied.Icon;
		delete copied.iconLabel;
		delete copied.size;
		delete copied.animate;
		delete copied.error;

		return { copied_props: copied as T, icon, btn };
	}, [props]);
}

type NativeButtonProps = Omit<ComponentProps<"button">, OmitProps>;
type ButtonProps = CustomButtonProps<NativeButtonProps>;

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
	(props, ref) => {
		const { copied_props, icon, btn } = useButtonProps(props);

		const { showBorder, animating } = useLinkAnimation(btn?.animate);

		return (
			<button
				{...copied_props}
				ref={ref}
				aria-label={icon.iconLabel}
				data-border={showBorder}
				data-animate={animating}
				data-error={btn.error}
				className={clsx(
					props.className,
					props.children && "pl-4",
					styles.button,
				)}
			>
				{copied_props.children}
				<div
					className={clsx(
						styles.icon,
						icon.size === "small" && "p-2",
					)}
				>
					<icon.Icon size={icon.size === "small" ? 12 : 16} />
				</div>
			</button>
		);
	},
);

type InternalLinkProps = CustomButtonProps<
	AnchorHTMLAttributes<HTMLAnchorElement>
>;

const InternalLink = React.forwardRef<HTMLAnchorElement, InternalLinkProps>(
	(props, ref) => {
		const { copied_props, icon, btn } = useButtonProps(props);

		const { showBorder, animating } = useLinkAnimation(btn?.animate);

		return (
			<a
				{...copied_props}
				ref={ref}
				aria-label={icon.iconLabel}
				data-border={showBorder}
				data-animate={animating}
				data-error={btn.error}
				className={clsx(
					styles.button,
					props.className,
					props.children && "pl-4",
				)}
			>
				{copied_props.children}
				<div
					className={clsx(
						styles.icon,
						icon.size === "small" && "p-2",
					)}
				>
					<icon.Icon size={icon.size === "small" ? 12 : 16} />
				</div>
			</a>
		);
	},
);

export const Link = createLink(InternalLink);

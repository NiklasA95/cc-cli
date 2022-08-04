import React from "react";
import { Story, Meta } from "@storybook/react";
import { documentationPath } from "@srcDS/storybook/constants";

import ProductReviews, {
  IProductReviews,
} from "@srcDS/components/organisms/ProductReviews";

export default {
  component: ProductReviews,
  title: `${documentationPath}/ProductReviews`,
  argTypes: {
    stdMarginTop: { table: { disable: true } },
    stdMarginBottom: { table: { disable: true } },
  },
} as Meta;

//TODO: Wrap component with decorators if needed
const StoryTpl: Story<IProductReviews> = (args) => <ProductReviews {...args} />;

export const DefaultStory = StoryTpl.bind({});
DefaultStory.storyName = "Default ProductReviews";
DefaultStory.args = {};

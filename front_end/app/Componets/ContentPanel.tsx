import { Tabs, UploadFile } from 'antd';
import React, { ReactNode, useState } from 'react';
import Common from '../Pages/Common';
import UserInfo from '../Pages/UserInfo';
import UserAdmin from '../Pages/UserAdmin';
import ModelAdmin from '../Pages/ModelAdmin';
import { NotificationInstance } from 'antd/es/notification/interface';

type ContentPanelItem = {
    label: string;
    key: string;
    disabled: boolean;
    children: ReactNode;
}

const generateItem = (label: string, key: string, disabled: boolean, children: ReactNode) => {
  return { label, key, disabled, children }
}

const ContentPanel: React.FC<{messageClient: NotificationInstance}> = (props) => {
  const { messageClient } = props;
  let Pages: Array<ContentPanelItem> = [
    generateItem("Main Page", "0", false, <Common messageClient={messageClient} />),
    generateItem("User Info", "1", false, <UserInfo />),
    generateItem("User Admin", "2", false, <UserAdmin />),
    generateItem("Model Admin", "3", false, <ModelAdmin />),
  ];
  return (
    <div>
      <Tabs
        defaultActiveKey="1"
        tabPosition={"left"}
        style={{ height: "100%" }}
        items={Pages}
      />
    </div>
  );
};

export default ContentPanel;
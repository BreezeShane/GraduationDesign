import React, { ReactNode, useEffect, useState } from 'react';
import { Button, Space, Table } from 'antd';
import type { TableColumnsType, TableProps } from 'antd';
import { NotificationInstance } from 'antd/es/notification/interface';
import axios from 'axios';

type TableRowSelection<T> = TableProps<T>['rowSelection'];

interface DataType {
  key: React.Key;
  username: string,
  useremail: string,
  contribution: number,
}

const FeedbackManage: React.FC<{messageClient: NotificationInstance}> = (props) => {
    const { messageClient } = props;
    const [selectedRowKeys, setSelectedRowKeys] = useState<React.Key[]>([]);
    const [feedbackList, setFeedbackList] = useState<DataType[]>([]);

    const onSelectChange = (newSelectedRowKeys: React.Key[]) => {
        setSelectedRowKeys(newSelectedRowKeys);
    };

    useEffect(() => {
      axios.get("/admin/feedback_manage",{
        params: {
          email: sessionStorage.getItem("useremail"),
        }
      }).then((res) => {
        let tfeedback = res.data;
        console.log(tfeedback);
        setFeedbackList(tfeedback);
      }).catch((err) => {
        console.log("get files error: ", err)
      });
    }, []);

    const rowSelection: TableRowSelection<DataType> = {
        selectedRowKeys,
        onChange: onSelectChange,
        selections: [
            Table.SELECTION_ALL,
            Table.SELECTION_INVERT,
            Table.SELECTION_NONE,
            {
                key: 'odd',
                text: 'Select Odd Row',
                onSelect: (changeableRowKeys) => {
                let newSelectedRowKeys = [];
                newSelectedRowKeys = changeableRowKeys.filter((_, index) => {
                    if (index % 2 !== 0) {
                    return false;
                    }
                    return true;
                });
                setSelectedRowKeys(newSelectedRowKeys);
                },
            },
            {
                key: 'even',
                text: 'Select Even Row',
                onSelect: (changeableRowKeys) => {
                let newSelectedRowKeys = [];
                newSelectedRowKeys = changeableRowKeys.filter((_, index) => {
                    if (index % 2 !== 0) {
                    return true;
                    }
                    return false;
                });
                setSelectedRowKeys(newSelectedRowKeys);
                },
            },
        ],
    };

    const handleOperation = (record: DataType) => {
      console.log(record.key); // 获取当前行的ID
      console.log(record.username); // 获取当前行的Name
    }

    const columns: TableColumnsType<DataType> = [
      {
        title: 'User Name',
        dataIndex: 'username',
      },
      {
        title: 'Email',
        dataIndex: 'useremail',
      },
      {
        title: 'Contributions',
        dataIndex: 'contribution',
      },
      {
        title: 'Operations',
        render: (text, record) => (
          <Space>
            <Button onClick={() => handleOperation(record)} style={{ background: "red", borderColor: "yellow" }}></Button>
            <Button onClick={() => handleOperation(record)} style={{ background: "red", borderColor: "yellow" }}></Button>
          </Space>
        ),
      }
    ];

    return <Table rowSelection={rowSelection} columns={columns} dataSource={feedbackList} />;
};

export default FeedbackManage;
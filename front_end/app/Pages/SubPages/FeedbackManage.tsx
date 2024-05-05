import React, { ReactNode, useEffect, useState } from 'react';
import { Button, Space, Table } from 'antd';
import type { TableColumnsType, TableProps } from 'antd';
import { NotificationInstance } from 'antd/es/notification/interface';
import axios from 'axios';

type TableRowSelection<T> = TableProps<T>['rowSelection'];

interface DataType {
  datetime: string,
  from_user_email: string,
  pic_link: string,
  real_label: string,
  submit_count: number,
  time_out: string,
  acceptable: boolean
}

interface DataOperate {
  pic_path: string,
  real_label: string,
  accept: boolean,
}

const columns: TableColumnsType<DataType> = [
  {
    title: 'Submission Datetime',
    dataIndex: 'datetime',
  },
  {
    title: 'User Email',
    dataIndex: 'from_user_email',
  },
  {
    title: 'Image Stored Location',
    dataIndex: 'pic_link',
  },
  {
    title: 'Expiration Datetime',
    dataIndex: 'time_out',
  },
  {
    title: 'Label',
    dataIndex: 'real_label',
  },
  {
    title: 'Count of Submission',
    dataIndex: 'submit_count',
  },
];

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

    const handleAccept = () => {
      let feedback2operate = feedbackList.filter((item) => {
        return selectedRowKeys.indexOf(item.datetime) > -1;
      });
      let data = [];
      for (let idx in feedback2operate) {
        data.push({
          pic_path: feedback2operate[idx].pic_link,
          real_label: feedback2operate[idx].real_label,
          acceptable: true,
        })
      }
      axios.post("/admin/feedback_manage", {
        useremail: sessionStorage.getItem("useremail"),
        files_to_operate: JSON.stringify(data)
      }).then((res) => {
        console.log(res);
      })
    }

    const handleReject = () => {
      let feedback2operate = feedbackList.filter((item) => {
        return selectedRowKeys.indexOf(item.datetime) > -1;
      });
      let data = [];
      for (let idx in feedback2operate) {
        data.push({
          pic_path: feedback2operate[idx].pic_link,
          real_label: feedback2operate[idx].real_label,
          acceptable: false,
        })
      }
      axios.post("/admin/feedback_manage", {
        useremail: sessionStorage.getItem("useremail"),
        files_to_operate: JSON.stringify(data)
      }).then((res) => {
        console.log(res);
      })
    }

    return (
      <>
        <Space>
            <Button onClick={handleAccept} size="large" style={{ background: "#3eb489", color: "#ffffff" }}>Accept</Button>
            <Button onClick={handleReject} size="large" style={{ background: "#FF2323", color: "#ffffff" }} >Reject</Button>
        </Space>
        <Table rowKey={item => item.datetime} rowSelection={rowSelection} columns={columns} dataSource={feedbackList} />
      </>
    );
};

export default FeedbackManage;